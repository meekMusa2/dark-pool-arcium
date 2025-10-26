# Architecture Deep Dive

## System Components

### 1. Client Layer (Frontend)
**Technology:** React + TypeScript  
**Responsibilities:**
- User interface
- Wallet connection
- Client-side encryption
- Order submission
- Result decryption

**Key Functions:**
```typescript
encryptOrder(orderData) → encryptedBlob
submitToSolana(encryptedBlob) → txSignature
decryptResult(encryptedMatch, key) → matchDetails
```

### 2. Solana Layer (Smart Contract)
**Technology:** Rust + Anchor Framework  
**Program ID:** 4kK1X4Dnxomgn858uCGvkfRtZhYd71r439hhz1ueNpF6

**Account Structure:**
```
DarkPool
├── authority: Pubkey
├── fee_basis_points: u16
├── total_volume: u64
└── active_orders: u32

Order
├── owner: Pubkey
├── encrypted_data: Vec<u8>      ← Arcium encrypted
├── status: OrderStatus
├── created_at: i64
└── compute_request_id: [u8; 32]

MatchingRequest
├── buy_orders: Vec<Pubkey>
├── sell_orders: Vec<Pubkey>
├── status: MatchingStatus
└── compute_request_id: [u8; 32]  ← Arcium job ID

TradeExecution
├── matching_request: Pubkey
├── encrypted_match_data: Vec<u8> ← Arcium result
├── arcium_signature: [u8; 64]
└── status: ExecutionStatus
```

### 3. Arcium Layer (MPC Network)
**Technology:** Multi-Party Computation  
**Function:** Private order matching

**MPC Configuration:**
```json
{
  "nodes": 5,
  "threshold": 3,
  "function": "match_orders",
  "privacy_level": "zero_knowledge"
}
```

**Matching Logic (Arcis Pseudocode):**
```rust
encrypted fn match_orders(
    encrypted_buys: Vec<EncryptedOrder>,
    encrypted_sells: Vec<EncryptedOrder>
) -> Vec<EncryptedMatch> {
    
    // Sort by price (on encrypted values)
    let sorted_buys = encrypted_sort_desc(encrypted_buys);
    let sorted_sells = encrypted_sort_asc(encrypted_sells);
    
    let mut matches = Vec::new();
    
    for buy in sorted_buys {
        for sell in sorted_sells {
            // Encrypted comparison
            if encrypted_gte(buy.price, sell.price) {
                let fill_qty = encrypted_min(buy.qty, sell.qty);
                let fill_price = encrypted_midpoint(buy.price, sell.price);
                
                matches.push(EncryptedMatch {
                    buy_id: buy.id,
                    sell_id: sell.id,
                    fill_price: seal_to(fill_price, [buy.owner, sell.owner]),
                    fill_qty: seal_to(fill_qty, [buy.owner, sell.owner]),
                });
                
                // Update quantities (encrypted)
                buy.qty = encrypted_sub(buy.qty, fill_qty);
                sell.qty = encrypted_sub(sell.qty, fill_qty);
                
                if encrypted_eq(buy.qty, 0) { break; }
            }
        }
    }
    
    return matches;
}
```

## Data Flow

### Order Submission Flow
```
1. User Input
   ↓
2. Client Encryption (Arcium SDK)
   {price: 100, qty: 10} → [0x4a2f...8bc3]
   ↓
3. Solana Transaction
   submit_order(encrypted_data)
   ↓
4. On-Chain Storage
   Order account created with encrypted blob
   ↓
5. Event Emission
   OrderSubmitted { order_id, owner, timestamp }
```

### Matching Flow
```
1. Matching Request
   request_matching([buy_ids], [sell_ids])
   ↓
2. Fetch Encrypted Orders
   Load encrypted_data from Solana accounts
   ↓
3. Submit to Arcium MPC
   POST /compute with encrypted orders
   ↓
4. MPC Computation
   5 nodes, 3-of-5 threshold
   Matching on encrypted data
   ↓
5. Result Encryption
   Seal results to matched parties
   ↓
6. Callback to Solana
   process_match_result(encrypted_matches, signature)
   ↓
7. Store Results
   TradeExecution account created
```

### Settlement Flow
```
1. Match Notification
   User receives encrypted match
   ↓
2. Decrypt Locally
   Use stored decryption key
   ↓
3. Approve Settlement
   User signs settlement transaction
   ↓
4. Token Transfer
   SPL token transfers executed
   ↓
5. Update State
   Order status → Settled
   Close accounts
```

## Privacy Analysis

### Threat Model

**Attacker Types:**
1. **Front-runner:** Sees order, tries to execute first
2. **MEV Bot:** Scans mempool for profitable opportunities
3. **Validator:** Has privileged access to transaction data
4. **Network Observer:** Monitors on-chain activity

**Defense Mechanisms:**

| Attack Vector | Traditional DEX | Dark Pool + Arcium |
|--------------|----------------|-------------------|
| Mempool snooping | ❌ Vulnerable | ✅ Protected (encrypted) |
| Front-running | ❌ Vulnerable | ✅ Protected (no price visible) |
| Sandwich attacks | ❌ Vulnerable | ✅ Protected (no quantity visible) |
| Validator MEV | ❌ Vulnerable | ✅ Protected (MPC execution) |
| Statistical analysis | ⚠️ Possible | ⚠️ Limited (encrypted) |
| Timing attacks | ⚠️ Possible | ✅ Mitigated (randomization) |

### Information Leakage Analysis

**What's Leaked:**
- Order exists (account created)
- Approximate gas cost (order size inference)
- Timing patterns (if not randomized)

**What's Protected:**
- Exact prices
- Exact quantities
- Buy/sell direction
- Link between orders
- Trading strategies

### Zero-Knowledge Properties

The system provides:
- **Completeness:** Valid matches always succeed
- **Soundness:** Invalid matches rejected
- **Zero-knowledge:** No information beyond match existence

## Performance Characteristics

### Latency Breakdown
```
Order Submission:    ~1 second    (Solana confirmation)
MPC Matching:        ~5 seconds   (Arcium computation)
Result Processing:   ~1 second    (Solana write)
Settlement:          ~1 second    (SPL transfer)
────────────────────────────────────────────────
Total:              ~8 seconds   (end-to-end)
```

### Scalability
- Orders/sec: ~1000 (Solana limit)
- Matches/batch: ~50-100 (MPC throughput)
- Concurrent users: ~10,000+ (with indexing)

### Cost Analysis
- Order submission: ~0.001 SOL (~$0.10)
- Matching request: ~0.002 SOL (~$0.20)
- Settlement: ~0.001 SOL (~$0.10)
- **Total per trade: ~$0.40** (vs $5-50 on Ethereum)

## Security Considerations

### Smart Contract Security
- ✅ No reentrancy vulnerabilities
- ✅ Integer overflow protection
- ✅ Access control on sensitive functions
- ✅ Input validation on all parameters

### Cryptographic Security
- ✅ Client-side encryption
- ✅ Secure key management
- ✅ Signature verification
- ✅ Replay attack protection

### MPC Security
- ✅ 3-of-5 threshold (Byzantine fault tolerant)
- ✅ No single point of failure
- ✅ Verifiable computation
- ✅ Collusion resistant (< 3 nodes)

## Comparison with Alternatives

| Feature | Public DEX | Private OTC | Our Dark Pool |
|---------|-----------|-------------|---------------|
| Privacy | ❌ None | ✅ Full | ✅ Full |
| Speed | ✅ Fast | ❌ Slow | ✅ Fast |
| Liquidity | ✅ High | ❌ Low | ⚠️ Growing |
| Decentralized | ✅ Yes | ❌ No | ✅ Yes |
| MEV Protection | ❌ No | ✅ Yes | ✅ Yes |
| Cost | ✅ Low | ❌ High | ✅ Low |

## Future Optimizations

### Short-term (1-3 months)
- [ ] Batch multiple matches in single MPC call
- [ ] Implement order book indexing
- [ ] Add order modification/cancellation
- [ ] Support multiple token pairs

### Medium-term (3-6 months)
- [ ] ZK proofs for match correctness
- [ ] Cross-chain atomic swaps
- [ ] Time-weighted average price (TWAP) orders
- [ ] Liquidity mining incentives

### Long-term (6+ months)
- [ ] Institutional custody integration
- [ ] Regulatory compliance features
- [ ] Advanced order types (iceberg, etc.)
- [ ] Machine learning for optimal matching
