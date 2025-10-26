import React, { useState, useEffect } from 'react';
import { Lock, ShieldCheck, Eye, EyeOff, TrendingUp, CheckCircle, AlertCircle } from 'lucide-react';

// Simulated types (replace with actual imports)
interface Order {
  id: string;
  price: number;
  quantity: number;
  side: 'buy' | 'sell';
  status: 'pending' | 'matching' | 'matched' | 'settled';
  encrypted: boolean;
  timestamp: number;
}

interface MatchResult {
  orderId: string;
  fillPrice: number;
  fillQuantity: number;
  counterparty: string;
}

export default function DarkPoolInterface() {
  const [orders, setOrders] = useState<Order[]>([]);
  const [orderForm, setOrderForm] = useState({
    price: '',
    quantity: '',
    side: 'buy' as 'buy' | 'sell'
  });
  const [showPrivacyInfo, setShowPrivacyInfo] = useState(false);
  const [matchResults, setMatchResults] = useState<MatchResult[]>([]);
  const [connected, setConnected] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Simulate wallet connection
  const connectWallet = () => {
    setConnected(true);
  };

  // Submit encrypted order
  const submitOrder = async () => {
    if (!orderForm.price || !orderForm.quantity) return;
    
    setIsSubmitting(true);
    
    // Simulate encryption and submission
    setTimeout(() => {
      const newOrder: Order = {
        id: 'order_' + Math.random().toString(36).substr(2, 9),
        price: parseFloat(orderForm.price),
        quantity: parseFloat(orderForm.quantity),
        side: orderForm.side,
        status: 'pending',
        encrypted: true,
        timestamp: Date.now()
      };
      
      setOrders(prev => [...prev, newOrder]);
      setOrderForm({ price: '', quantity: '', side: 'buy' });
      setIsSubmitting(false);
      
      // Simulate matching after 3 seconds
      setTimeout(() => {
        setOrders(prev => prev.map(o => 
          o.id === newOrder.id ? { ...o, status: 'matching' } : o
        ));
        
        // Simulate match found
        setTimeout(() => {
          setOrders(prev => prev.map(o => 
            o.id === newOrder.id ? { ...o, status: 'matched' } : o
          ));
          
          setMatchResults(prev => [...prev, {
            orderId: newOrder.id,
            fillPrice: newOrder.price,
            fillQuantity: newOrder.quantity,
            counterparty: 'Hidden'
          }]);
        }, 2000);
      }, 3000);
    }, 1000);
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 text-white p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="bg-purple-600 p-3 rounded-lg">
                <Lock className="w-8 h-8" />
              </div>
              <div>
                <h1 className="text-3xl font-bold">Private Dark Pool</h1>
                <p className="text-purple-300">Powered by Arcium Encrypted Compute</p>
              </div>
            </div>
            <button
              onClick={connectWallet}
              className={`px-6 py-3 rounded-lg font-semibold transition-all ${
                connected
                  ? 'bg-green-600 hover:bg-green-700'
                  : 'bg-purple-600 hover:bg-purple-700'
              }`}
            >
              {connected ? '✓ Wallet Connected' : 'Connect Wallet'}
            </button>
          </div>
        </div>

        {/* Privacy Banner */}
        <div className="bg-purple-900/40 border border-purple-500/30 rounded-xl p-4 mb-6">
          <div className="flex items-start gap-3">
            <ShieldCheck className="w-6 h-6 text-green-400 flex-shrink-0 mt-1" />
            <div className="flex-1">
              <h3 className="font-semibold text-lg mb-1">Zero-Trust Privacy Enabled</h3>
              <p className="text-purple-200 text-sm">
                All orders are encrypted client-side using Arcium's MPC network. 
                Your trading data remains confidential - no front-running, no MEV exploitation.
              </p>
              <button
                onClick={() => setShowPrivacyInfo(!showPrivacyInfo)}
                className="text-purple-300 text-sm mt-2 hover:text-purple-100 flex items-center gap-1"
              >
                {showPrivacyInfo ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                {showPrivacyInfo ? 'Hide' : 'Show'} Privacy Details
              </button>
            </div>
          </div>
          
          {showPrivacyInfo && (
            <div className="mt-4 grid md:grid-cols-2 gap-4 text-sm">
              <div className="bg-red-900/20 border border-red-500/30 rounded-lg p-3">
                <h4 className="font-semibold text-red-300 mb-2">❌ What Attackers CANNOT See:</h4>
                <ul className="text-red-200 space-y-1">
                  <li>• Your order price</li>
                  <li>• Your order quantity</li>
                  <li>• Buy or sell direction</li>
                  <li>• Your trading strategy</li>
                  <li>• Connection to other orders</li>
                </ul>
              </div>
              <div className="bg-green-900/20 border border-green-500/30 rounded-lg p-3">
                <h4 className="font-semibold text-green-300 mb-2">✓ What IS Public:</h4>
                <ul className="text-green-200 space-y-1">
                  <li>• Order exists (encrypted blob)</li>
                  <li>• Submission timestamp</li>
                  <li>• Your wallet address</li>
                  <li>• Settlement transaction (post-match)</li>
                </ul>
              </div>
            </div>
          )}
        </div>

        <div className="grid lg:grid-cols-3 gap-6">
          {/* Order Entry */}
          <div className="lg:col-span-1">
            <div className="bg-slate-800/50 rounded-xl p-6 border border-slate-700">
              <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
                <Lock className="w-5 h-5 text-purple-400" />
                Submit Encrypted Order
              </h2>
              
              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-slate-300 mb-2">Order Side</label>
                  <div className="grid grid-cols-2 gap-2">
                    <button
                      onClick={() => setOrderForm(prev => ({ ...prev, side: 'buy' }))}
                      className={`py-3 rounded-lg font-semibold transition-all ${
                        orderForm.side === 'buy'
                          ? 'bg-green-600 text-white'
                          : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                      }`}
                    >
                      Buy
                    </button>
                    <button
                      onClick={() => setOrderForm(prev => ({ ...prev, side: 'sell' }))}
                      className={`py-3 rounded-lg font-semibold transition-all ${
                        orderForm.side === 'sell'
                          ? 'bg-red-600 text-white'
                          : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                      }`}
                    >
                      Sell
                    </button>
                  </div>
                </div>

                <div>
                  <label className="block text-sm text-slate-300 mb-2">Price (USDC)</label>
                  <input
                    type="number"
                    value={orderForm.price}
                    onChange={(e) => setOrderForm(prev => ({ ...prev, price: e.target.value }))}
                    placeholder="100.50"
                    className="w-full bg-slate-700 border border-slate-600 rounded-lg px-4 py-3 text-white focus:outline-none focus:border-purple-500"
                  />
                  <div className="flex items-center gap-1 mt-1 text-xs text-purple-300">
                    <Lock className="w-3 h-3" />
                    <span>Price will be encrypted</span>
                  </div>
                </div>

                <div>
                  <label className="block text-sm text-slate-300 mb-2">Quantity (SOL)</label>
                  <input
                    type="number"
                    value={orderForm.quantity}
                    onChange={(e) => setOrderForm(prev => ({ ...prev, quantity: e.target.value }))}
                    placeholder="10.0"
                    className="w-full bg-slate-700 border border-slate-600 rounded-lg px-4 py-3 text-white focus:outline-none focus:border-purple-500"
                  />
                  <div className="flex items-center gap-1 mt-1 text-xs text-purple-300">
                    <Lock className="w-3 h-3" />
                    <span>Quantity will be encrypted</span>
                  </div>
                </div>

                <button
                  onClick={submitOrder}
                  disabled={!connected || isSubmitting || !orderForm.price || !orderForm.quantity}
                  className="w-full bg-purple-600 hover:bg-purple-700 disabled:bg-slate-700 disabled:text-slate-500 py-3 rounded-lg font-semibold transition-all"
                >
                  {isSubmitting ? 'Encrypting & Submitting...' : 'Submit Encrypted Order'}
                </button>
              </div>
            </div>

            {/* Stats */}
            <div className="bg-slate-800/50 rounded-xl p-6 border border-slate-700 mt-6">
              <h3 className="text-lg font-bold mb-4">Privacy Stats</h3>
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <span className="text-slate-300">Orders Encrypted</span>
                  <span className="font-bold text-green-400">{orders.length}</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-slate-300">Matches Found</span>
                  <span className="font-bold text-purple-400">{matchResults.length}</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-slate-300">Front-Running Attempts</span>
                  <span className="font-bold text-red-400">0 (Blocked)</span>
                </div>
              </div>
            </div>
          </div>

          {/* Order Book & Matches */}
          <div className="lg:col-span-2 space-y-6">
            {/* Your Orders */}
            <div className="bg-slate-800/50 rounded-xl p-6 border border-slate-700">
              <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
                <TrendingUp className="w-5 h-5 text-purple-400" />
                Your Encrypted Orders
              </h2>
              
              {orders.length === 0 ? (
                <div className="text-center py-12 text-slate-400">
                  <Lock className="w-12 h-12 mx-auto mb-3 opacity-50" />
                  <p>No orders submitted yet</p>
                  <p className="text-sm mt-1">Submit your first encrypted order to get started</p>
                </div>
              ) : (
                <div className="space-y-3">
                  {orders.map(order => (
                    <div
                      key={order.id}
                      className="bg-slate-700/50 rounded-lg p-4 border border-slate-600"
                    >
                      <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                          <span className={`px-3 py-1 rounded-full text-xs font-semibold ${
                            order.side === 'buy'
                              ? 'bg-green-600/20 text-green-300'
                              : 'bg-red-600/20 text-red-300'
                          }`}>
                            {order.side.toUpperCase()}
                          </span>
                          <span className="text-xs text-slate-400">
                            {new Date(order.timestamp).toLocaleTimeString()}
                          </span>
                        </div>
                        <StatusBadge status={order.status} />
                      </div>
                      
                      <div className="grid grid-cols-2 gap-3 text-sm">
                        <div>
                          <div className="text-slate-400 text-xs mb-1">Price</div>
                          <div className="flex items-center gap-1">
                            <Lock className="w-3 h-3 text-purple-400" />
                            <span className="font-mono">${order.price.toFixed(2)}</span>
                          </div>
                        </div>
                        <div>
                          <div className="text-slate-400 text-xs mb-1">Quantity</div>
                          <div className="flex items-center gap-1">
                            <Lock className="w-3 h-3 text-purple-400" />
                            <span className="font-mono">{order.quantity} SOL</span>
                          </div>
                        </div>
                      </div>
                      
                      {order.status === 'matching' && (
                        <div className="mt-3 text-xs text-purple-300 flex items-center gap-2">
                          <div className="animate-spin rounded-full h-3 w-3 border-b-2 border-purple-400"></div>
                          <span>Matching via Arcium MPC network...</span>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Match Results */}
            {matchResults.length > 0 && (
              <div className="bg-slate-800/50 rounded-xl p-6 border border-slate-700">
                <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
                  <CheckCircle className="w-5 h-5 text-green-400" />
                  Your Match Results
                </h2>
                
                <div className="space-y-3">
                  {matchResults.map((match, idx) => (
                    <div
                      key={idx}
                      className="bg-green-900/20 border border-green-500/30 rounded-lg p-4"
                    >
                      <div className="flex items-center gap-2 mb-3">
                        <CheckCircle className="w-5 h-5 text-green-400" />
                        <span className="font-semibold text-green-300">Match Found!</span>
                      </div>
                      
                      <div className="grid grid-cols-3 gap-4 text-sm">
                        <div>
                          <div className="text-green-200/60 text-xs mb-1">Fill Price</div>
                          <div className="font-mono text-green-300">${match.fillPrice.toFixed(2)}</div>
                        </div>
                        <div>
                          <div className="text-green-200/60 text-xs mb-1">Fill Quantity</div>
                          <div className="font-mono text-green-300">{match.fillQuantity} SOL</div>
                        </div>
                        <div>
                          <div className="text-green-200/60 text-xs mb-1">Counterparty</div>
                          <div className="flex items-center gap-1">
                            <Lock className="w-3 h-3 text-green-400" />
                            <span className="text-green-300">{match.counterparty}</span>
                          </div>
                        </div>
                      </div>
                      
                      <button className="w-full mt-3 bg-green-600 hover:bg-green-700 py-2 rounded-lg text-sm font-semibold transition-all">
                        Settle Trade
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* How It Works */}
            <div className="bg-gradient-to-r from-purple-900/40 to-blue-900/40 rounded-xl p-6 border border-purple-500/30">
              <h3 className="text-lg font-bold mb-3 flex items-center gap-2">
                <AlertCircle className="w-5 h-5" />
                How Privacy Works
              </h3>
              <div className="space-y-2 text-sm text-purple-100">
                <div className="flex gap-2">
                  <span className="text-purple-300">1.</span>
                  <span>Orders encrypted client-side before submission to Solana</span>
                </div>
                <div className="flex gap-2">
                  <span className="text-purple-300">2.</span>
                  <span>Arcium MPC network performs matching on encrypted data</span>
                </div>
                <div className="flex gap-2">
                  <span className="text-purple-300">3.</span>
                  <span>Results sealed to matched parties only - counterparty remains hidden</span>
                </div>
                <div className="flex gap-2">
                  <span className="text-purple-300">4.</span>
                  <span>Settlement executes on-chain with decrypted values post-match</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function StatusBadge({ status }: { status: Order['status'] }) {
  const configs = {
    pending: { bg: 'bg-yellow-600/20', text: 'text-yellow-300', label: 'Pending' },
    matching: { bg: 'bg-blue-600/20', text: 'text-blue-300', label: 'Matching' },
    matched: { bg: 'bg-green-600/20', text: 'text-green-300', label: 'Matched' },
    settled: { bg: 'bg-purple-600/20', text: 'text-purple-300', label: 'Settled' },
  };
  
  const config = configs[status];
  
  return (
    <span className={`px-3 py-1 rounded-full text-xs font-semibold ${config.bg} ${config.text}`}>
      {config.label}
    </span>
  );
}
