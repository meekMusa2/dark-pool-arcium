use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("4kK1X4Dnxomgn858uCGvkfRtZhYd71r439hhz1ueNpF6");

#[program]
pub mod dark_pool {
    use super::*;

    /// Initialize the dark pool
    pub fn initialize(ctx: Context<Initialize>, fee_bps: u16) -> Result<()> {
        let pool = &mut ctx.accounts.dark_pool;
        pool.authority = ctx.accounts.authority.key();
        pool.fee_basis_points = fee_bps;
        pool.total_volume = 0;
        pool.active_orders = 0;
        pool.bump = ctx.bumps.dark_pool;
        
        msg!("Dark pool initialized with fee: {} bps", fee_bps);
        Ok(())
    }

    /// Submit an encrypted order to the dark pool
    pub fn submit_order(
        ctx: Context<SubmitOrder>,
        encrypted_order_data: Vec<u8>,
        order_side: OrderSide,
    ) -> Result<()> {
        require!(encrypted_order_data.len() > 0, DarkPoolError::EmptyOrderData);
        require!(encrypted_order_data.len() <= 512, DarkPoolError::OrderDataTooLarge);

        let order = &mut ctx.accounts.order;
        let pool = &mut ctx.accounts.dark_pool;
        let clock = Clock::get()?;

        // Store order details
        order.owner = ctx.accounts.owner.key();
        order.encrypted_data = encrypted_order_data;
        order.order_side = order_side;
        order.status = OrderStatus::Submitted;
        order.created_at = clock.unix_timestamp;
        order.updated_at = clock.unix_timestamp;
        order.compute_request_id = [0u8; 32]; // Will be set when matching starts
        order.bump = ctx.bumps.order;

        pool.active_orders += 1;

        emit!(OrderSubmitted {
            order_id: order.key(),
            owner: order.owner,
            timestamp: order.created_at,
        });

        msg!("Order submitted: {}", order.key());
        Ok(())
    }

    /// Request matching through Arcium MPC
    pub fn request_matching(
        ctx: Context<RequestMatching>,
        buy_orders: Vec<Pubkey>,
        sell_orders: Vec<Pubkey>,
    ) -> Result<()> {
        require!(buy_orders.len() > 0, DarkPoolError::NoOrders);
        require!(sell_orders.len() > 0, DarkPoolError::NoOrders);

        let matching_request = &mut ctx.accounts.matching_request;
        let clock = Clock::get()?;

        // Store matching request details
        matching_request.buy_orders = buy_orders.clone();
        matching_request.sell_orders = sell_orders.clone();
        matching_request.status = MatchingStatus::Pending;
        matching_request.requested_at = clock.unix_timestamp;
        matching_request.compute_request_id = [0u8; 32]; // Set by Arcium callback
        matching_request.bump = ctx.bumps.matching_request;

        // In production, this would call Arcium SDK to initiate MPC compute
        // For now, we emit an event that an off-chain service will pick up
        emit!(MatchingRequested {
            request_id: matching_request.key(),
            num_buy_orders: matching_request.buy_orders.len() as u32,
            num_sell_orders: matching_request.sell_orders.len() as u32,
            timestamp: matching_request.requested_at,
        });

        msg!("Matching requested for {} buy and {} sell orders", 
             buy_orders.len(), sell_orders.len());
        Ok(())
    }

    /// Process match result from Arcium
    pub fn process_match_result(
        ctx: Context<ProcessMatchResult>,
        encrypted_matches: Vec<u8>,
        arcium_signature: [u8; 64],
    ) -> Result<()> {
        let matching_request = &mut ctx.accounts.matching_request;
        let trade_execution = &mut ctx.accounts.trade_execution;
        let clock = Clock::get()?;

        // Verify this is a valid Arcium callback
        // In production: verify arcium_signature against known Arcium public key
        require!(
            matching_request.status == MatchingStatus::Pending,
            DarkPoolError::InvalidMatchingState
        );

        // Store the encrypted match results
        trade_execution.matching_request = matching_request.key();
        trade_execution.encrypted_match_data = encrypted_matches;
        trade_execution.arcium_signature = arcium_signature;
        trade_execution.status = ExecutionStatus::Matched;
        trade_execution.matched_at = clock.unix_timestamp;
        trade_execution.bump = ctx.bumps.trade_execution;

        matching_request.status = MatchingStatus::Completed;

        emit!(MatchCompleted {
            request_id: matching_request.key(),
            execution_id: trade_execution.key(),
            timestamp: trade_execution.matched_at,
        });

        msg!("Match result processed: {}", trade_execution.key());
        Ok(())
    }

    /// Settle a matched trade
    pub fn settle_trade(
        ctx: Context<SettleTrade>,
        fill_amount: u64,
    ) -> Result<()> {
        let trade_execution = &mut ctx.accounts.trade_execution;
        let pool = &mut ctx.accounts.dark_pool;
        
        require!(
            trade_execution.status == ExecutionStatus::Matched,
            DarkPoolError::TradeNotMatched
        );

        // Calculate fee
        let fee_amount = (fill_amount as u128)
            .checked_mul(pool.fee_basis_points as u128)
            .unwrap()
            .checked_div(10000)
            .unwrap() as u64;
        
        let transfer_amount = fill_amount.checked_sub(fee_amount).unwrap();

        // Transfer tokens from buyer to seller
        let cpi_accounts = Transfer {
            from: ctx.accounts.buyer_token_account.to_account_info(),
            to: ctx.accounts.seller_token_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, transfer_amount)?;

        // Transfer fee to pool
        if fee_amount > 0 {
            let cpi_accounts_fee = Transfer {
                from: ctx.accounts.buyer_token_account.to_account_info(),
                to: ctx.accounts.fee_account.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            };
            let cpi_ctx_fee = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts_fee
            );
            token::transfer(cpi_ctx_fee, fee_amount)?;
        }

        trade_execution.status = ExecutionStatus::Settled;
        pool.total_volume += fill_amount;

        emit!(TradeSettled {
            execution_id: trade_execution.key(),
            amount: transfer_amount,
            fee: fee_amount,
        });

        msg!("Trade settled: {} tokens transferred", transfer_amount);
        Ok(())
    }

    /// Cancel an order
    pub fn cancel_order(ctx: Context<CancelOrder>) -> Result<()> {
        let order = &mut ctx.accounts.order;
        let pool = &mut ctx.accounts.dark_pool;
        
        require!(
            order.status == OrderStatus::Submitted,
            DarkPoolError::CannotCancelOrder
        );

        order.status = OrderStatus::Cancelled;
        pool.active_orders = pool.active_orders.saturating_sub(1);

        emit!(OrderCancelled {
            order_id: order.key(),
            owner: order.owner,
        });

        Ok(())
    }
}

// ===== ACCOUNTS =====

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + DarkPool::INIT_SPACE,
        seeds = [b"dark_pool"],
        bump
    )]
    pub dark_pool: Account<'info, DarkPool>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitOrder<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Order::INIT_SPACE,
        seeds = [b"order", owner.key().as_ref(), &[dark_pool.active_orders as u8]],
        bump
    )]
    pub order: Account<'info, Order>,
    
    #[account(mut)]
    pub dark_pool: Account<'info, DarkPool>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RequestMatching<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + MatchingRequest::INIT_SPACE,
        seeds = [b"matching", authority.key().as_ref()],
        bump
    )]
    pub matching_request: Account<'info, MatchingRequest>,
    
    pub dark_pool: Account<'info, DarkPool>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProcessMatchResult<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + TradeExecution::INIT_SPACE,
        seeds = [b"execution", matching_request.key().as_ref()],
        bump
    )]
    pub trade_execution: Account<'info, TradeExecution>,
    
    #[account(mut)]
    pub matching_request: Account<'info, MatchingRequest>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SettleTrade<'info> {
    #[account(mut)]
    pub trade_execution: Account<'info, TradeExecution>,
    
    pub dark_pool: Account<'info, DarkPool>,
    
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub fee_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(
        mut,
        has_one = owner,
    )]
    pub order: Account<'info, Order>,
    
    #[account(mut)]
    pub dark_pool: Account<'info, DarkPool>,
    
    pub owner: Signer<'info>,
}

// ===== STATE =====

#[account]
#[derive(InitSpace)]
pub struct DarkPool {
    pub authority: Pubkey,
    pub fee_basis_points: u16,
    pub total_volume: u64,
    pub active_orders: u32,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Order {
    pub owner: Pubkey,
    #[max_len(512)]
    pub encrypted_data: Vec<u8>,
    pub order_side: OrderSide,
    pub status: OrderStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub compute_request_id: [u8; 32],
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct MatchingRequest {
    #[max_len(50)]
    pub buy_orders: Vec<Pubkey>,
    #[max_len(50)]
    pub sell_orders: Vec<Pubkey>,
    pub status: MatchingStatus,
    pub requested_at: i64,
    pub compute_request_id: [u8; 32],
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct TradeExecution {
    pub matching_request: Pubkey,
    #[max_len(1024)]
    pub encrypted_match_data: Vec<u8>,
    pub arcium_signature: [u8; 64],
    pub status: ExecutionStatus,
    pub matched_at: i64,
    pub bump: u8,
}

// ===== ENUMS =====

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum OrderStatus {
    Submitted,
    InMatching,
    Matched,
    Executed,
    Settled,
    Cancelled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum MatchingStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum ExecutionStatus {
    Matched,
    Settled,
}

// ===== EVENTS =====

#[event]
pub struct OrderSubmitted {
    pub order_id: Pubkey,
    pub owner: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct MatchingRequested {
    pub request_id: Pubkey,
    pub num_buy_orders: u32,
    pub num_sell_orders: u32,
    pub timestamp: i64,
}

#[event]
pub struct MatchCompleted {
    pub request_id: Pubkey,
    pub execution_id: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct TradeSettled {
    pub execution_id: Pubkey,
    pub amount: u64,
    pub fee: u64,
}

#[event]
pub struct OrderCancelled {
    pub order_id: Pubkey,
    pub owner: Pubkey,
}

// ===== ERRORS =====

#[error_code]
pub enum DarkPoolError {
    #[msg("Order data cannot be empty")]
    EmptyOrderData,
    #[msg("Order data exceeds maximum size")]
    OrderDataTooLarge,
    #[msg("No orders provided for matching")]
    NoOrders,
    #[msg("Invalid matching state")]
    InvalidMatchingState,
    #[msg("Trade not matched yet")]
    TradeNotMatched,
    #[msg("Cannot cancel order in current state")]
    CannotCancelOrder,
}
