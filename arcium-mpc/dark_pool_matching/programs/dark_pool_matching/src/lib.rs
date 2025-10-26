use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

const COMP_DEF_OFFSET_MATCH_ORDER: u32 = comp_def_offset("match_single_order");

declare_id!("9KBfeudVctErFqnQEv8tZcNfTkhSP33XQE8FH7NvMnSb");

#[arcium_program]
pub mod dark_pool_matching {
    use super::*;

    pub fn init_match_order_comp_def(ctx: Context<InitMatchOrderCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }

    pub fn match_single_order(
        ctx: Context<MatchSingleOrder>,
        computation_offset: u64,
        buy_price: [u8; 32],
        buy_quantity: [u8; 32],
        sell_price: [u8; 32],
        sell_quantity: [u8; 32],
        pub_key: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        let args = vec![
            Argument::ArcisPubkey(pub_key),
            Argument::PlaintextU128(nonce),
            Argument::EncryptedU64(buy_price),
            Argument::EncryptedU64(buy_quantity),
            Argument::EncryptedU64(sell_price),
            Argument::EncryptedU64(sell_quantity),
        ];

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            None,
            vec![MatchSingleOrderCallback::callback_ix(&[])],
        )?;

        Ok(())
    }

    #[arcium_callback(encrypted_ix = "match_single_order")]
    pub fn match_single_order_callback(
        ctx: Context<MatchSingleOrderCallback>,
        output: ComputationOutputs<MatchSingleOrderOutput>,
    ) -> Result<()> {
        let result = match output {
            ComputationOutputs::Success(MatchSingleOrderOutput { field_0 }) => field_0,
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        emit!(MatchEvent {
            matched: result.ciphertexts[0],
            fill_price: result.ciphertexts[1],
            fill_quantity: result.ciphertexts[2],
            nonce: result.nonce.to_le_bytes(),
        });
        Ok(())
    }
}

#[queue_computation_accounts("match_single_order", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct MatchSingleOrder<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, SignerAccount>,
    #[account(
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(
        mut,
        address = derive_mempool_pda!()
    )]
    /// CHECK: mempool_account, checked by the arcium program.
    pub mempool_account: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_execpool_pda!()
    )]
    /// CHECK: executing_pool, checked by the arcium program.
    pub executing_pool: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_comp_pda!(computation_offset)
    )]
    /// CHECK: computation_account, checked by the arcium program.
    pub computation_account: UncheckedAccount<'info>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH_ORDER)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(
        mut,
        address = derive_cluster_pda!(mxe_account)
    )]
    pub cluster_account: Account<'info, Cluster>,
    #[account(
        mut,
        address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS,
    )]
    pub pool_account: Account<'info, FeePool>,
    #[account(
        address = ARCIUM_CLOCK_ACCOUNT_ADDRESS
    )]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

#[callback_accounts("match_single_order")]
#[derive(Accounts)]
pub struct MatchSingleOrderCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_MATCH_ORDER)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
}

#[init_computation_definition_accounts("match_single_order", payer)]
#[derive(Accounts)]
pub struct InitMatchOrderCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    /// Can't check it here as it's not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct MatchEvent {
    pub matched: [u8; 32],
    pub fill_price: [u8; 32],
    pub fill_quantity: [u8; 32],
    pub nonce: [u8; 16],
}

#[error_code]
pub enum ErrorCode {
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("Cluster not set")]
    ClusterNotSet,
}
