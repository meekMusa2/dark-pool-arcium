use arcis_imports::*;

#[encrypted]
mod circuits {
    use arcis_imports::*;

    // Order structure for encrypted orders
    pub struct Order {
        pub price: u64,      // Price in USDC (scaled by 1e6)
        pub quantity: u64,   // Quantity in tokens
        pub is_buy: bool,    // true for buy, false for sell
    }

    // Match result structure
    pub struct MatchResult {
        pub matched: bool,
        pub fill_price: u64,
        pub fill_quantity: u64,
    }

    // Simple order matching: match if buy price >= sell price
    #[instruction]
    pub fn match_single_order(
        buy_order: Enc<Shared, Order>,
        sell_order: Enc<Shared, Order>,
    ) -> Enc<Shared, MatchResult> {
        let buy = buy_order.to_arcis();
        let sell = sell_order.to_arcis();

        // Check if orders can match: buy_price >= sell_price
        let can_match = buy.price >= sell.price;

        let result = if can_match {
            // Calculate fill price (midpoint)
            let fill_price = (buy.price + sell.price) / 2;
            
            // Calculate fill quantity (minimum of both)
            let fill_quantity = if buy.quantity < sell.quantity {
                buy.quantity
            } else {
                sell.quantity
            };

            MatchResult {
                matched: true,
                fill_price,
                fill_quantity,
            }
        } else {
            MatchResult {
                matched: false,
                fill_price: 0,
                fill_quantity: 0,
            }
        };

        buy_order.owner.from_arcis(result)
    }
}
