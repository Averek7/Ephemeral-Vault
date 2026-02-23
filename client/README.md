# Ephemeral Vault - Frontend Design Map

## 🎯 Product Overview

**Ephemeral Vault** is a Solana-based temporary trading authorization system that allows users to:
1. Pre-approve a spending limit for automated trading
2. Delegate trading permissions to a bot/service for a limited time
3. Monitor trades in real-time
4. Revoke access instantly

---

## 📱 User Flows

### Primary Flow: First-Time Setup
```
Landing Page → Connect Wallet → Create Vault → Approve Delegate → Fund Vault → Monitor Dashboard
```

### Secondary Flows:
- **Returning User**: Dashboard → Fund/Withdraw → Monitor
- **Emergency**: Dashboard → Emergency Pause → Revoke Access
- **Session Renewal**: Dashboard → Extend Session (when near expiry)

---

## 🎨 Page Structure & Components

### 1. **Landing Page** (`/`)

#### Hero Section
```
┌─────────────────────────────────────────────────┐
│  [Logo]                    [Connect Wallet Btn] │
├─────────────────────────────────────────────────┤
│                                                 │
│         Secure Temporary Trading Access         │
│         on Solana                               │
│                                                 │
│    Pre-approve spending limits for automated    │
│    trading with time-based session control      │
│                                                 │
│         [Get Started] [Watch Demo]              │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Key Elements:**
- Animated background (subtle Solana-themed particles)
- Value propositions (3 columns):
  - 🔒 **Time-Limited Access**: Sessions expire automatically
  - 💰 **Spending Caps**: Pre-approved limits you control
  - ⚡ **Instant Revoke**: Take back control anytime

**Components:**
- `<HeroSection />`
- `<ValuePropositions />`
- `<WalletConnectButton />`

---

### 2. **Dashboard** (`/dashboard`)

This is the main control center after vault creation.

```
┌─────────────────────────────────────────────────────────────┐
│ [Logo]  Dashboard        [Network: Mainnet ▼] [@user...abc] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ Vault Overview                    [Status: 🟢 Active] │  │
│  ├─────────────────────────────────────────────────────┤  │
│  │  Available Balance:    2.5 SOL                      │  │
│  │  Approved Limit:       10.0 SOL                     │  │
│  │  Total Deposited:      5.0 SOL                      │  │
│  │  Total Withdrawn:      2.5 SOL                      │  │
│  │  Trades Executed:      23                           │  │
│  │                                                      │  │
│  │  [Deposit] [Withdraw] [Emergency Pause]             │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ Session Management                                   │  │
│  ├─────────────────────────────────────────────────────┤  │
│  │  Delegate: 8xKd...m9Pq                              │  │
│  │  Status: Active (⏱️ Expires in 45 min)              │  │
│  │  Last Activity: 2 min ago                           │  │
│  │                                                      │  │
│  │  [Renew Session] [Revoke Access]                    │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ Recent Trades                      [View All]        │  │
│  ├──────────┬──────────┬──────────┬─────────┬─────────┤  │
│  │ Time     │ Type     │ Amount   │ Fee     │ Status  │  │
│  ├──────────┼──────────┼──────────┼─────────┼─────────┤  │
│  │ 2m ago   │ Swap     │ 0.5 SOL  │ 0.001   │ ✅      │  │
│  │ 5m ago   │ Swap     │ 0.3 SOL  │ 0.001   │ ✅      │  │
│  │ 12m ago  │ Swap     │ 1.0 SOL  │ 0.002   │ ✅      │  │
│  └──────────┴──────────┴──────────┴─────────┴─────────┘  │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ Activity Chart (Last 24h)                            │  │
│  │  [Line chart showing trades over time]               │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Key Components:**
- `<VaultOverview />` - Shows balances, limits, stats
- `<SessionManager />` - Delegate info, countdown timer, controls
- `<TradeHistory />` - Real-time trade list with filters
- `<ActivityChart />` - Visual representation of trading activity
- `<QuickActions />` - Deposit, Withdraw, Pause buttons

**State Management:**
```typescript
interface DashboardState {
  vaultData: VaultAccount | null;
  trades: Trade[];
  sessionStatus: SessionStatus;
  isLoading: boolean;
  error: string | null;
}
```

---

### 3. **Create Vault Flow** (`/create`)

#### Step 1: Set Approved Amount
```
┌─────────────────────────────────────────┐
│  Create Your Ephemeral Vault            │
│  ────────────────────────────            │
│                                          │
│  Step 1 of 3: Set Spending Limit        │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │  Approved Amount                   │ │
│  │  ┌──────────────────────────────┐  │ │
│  │  │  10        [SOL ▼]           │  │ │
│  │  └──────────────────────────────┘  │ │
│  │                                    │ │
│  │  This is the maximum amount you    │ │
│  │  authorize for trading.            │ │
│  │                                    │ │
│  │  Min: 0.001 SOL | Max: 1000 SOL   │ │
│  └────────────────────────────────────┘ │
│                                          │
│         [Cancel]  [Next Step →]          │
└─────────────────────────────────────────┘
```

#### Step 2: Approve Delegate
```
┌─────────────────────────────────────────┐
│  Create Your Ephemeral Vault            │
│  ────────────────────────────────        │
│                                          │
│  Step 2 of 3: Delegate Trading Bot      │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │  Delegate Wallet Address           │ │
│  │  ┌──────────────────────────────┐  │ │
│  │  │  8xKd...m9Pq                 │  │ │
│  │  └──────────────────────────────┘  │ │
│  │                                    │ │
│  │  Session Duration                  │ │
│  │  ◉ 1 Hour (recommended)            │ │
│  │  ○ 30 Minutes                      │ │
│  │  ○ Custom: [____] minutes          │ │
│  │                                    │ │
│  │  ⚠️ Only approve trusted bots!     │ │
│  └────────────────────────────────────┘ │
│                                          │
│         [← Back]  [Next Step →]          │
└─────────────────────────────────────────┘
```

#### Step 3: Initial Deposit
```
┌─────────────────────────────────────────┐
│  Create Your Ephemeral Vault            │
│  ────────────────────────────────        │
│                                          │
│  Step 3 of 3: Initial Deposit           │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │  Deposit Amount                    │ │
│  │  ┌──────────────────────────────┐  │ │
│  │  │  2.5      [SOL]    [Max]     │  │ │
│  │  └──────────────────────────────┘  │ │
│  │                                    │ │
│  │  Wallet Balance: 5.2 SOL           │ │
│  │  Approved Limit: 10.0 SOL          │ │
│  │                                    │ │
│  │  Transaction Fee: ~0.001 SOL       │ │
│  └────────────────────────────────────┘ │
│                                          │
│  Summary:                                │
│  • Approved: 10 SOL                     │
│  • Depositing: 2.5 SOL                  │
│  • Delegate: 8xKd...m9Pq                │
│  • Expires: In 1 hour                   │
│                                          │
│         [← Back]  [Create Vault]         │
└─────────────────────────────────────────┘
```

**Components:**
- `<CreateVaultWizard />` - Multi-step form container
- `<ApprovedAmountInput />` - With validation and slider
- `<DelegateSelector />` - Address input with verification
- `<DepositForm />` - Amount input with balance checks
- `<TransactionConfirmation />` - Summary before signing

---

### 4. **Modals & Dialogs**

#### Emergency Pause Modal
```
┌─────────────────────────────────────────┐
│  ⚠️ Emergency Pause                      │
├─────────────────────────────────────────┤
│                                          │
│  This will immediately:                  │
│  • Stop all trading activity            │
│  • Block new deposits                   │
│  • Prevent delegate from executing      │
│                                          │
│  Your funds remain safe in the vault.   │
│                                          │
│  You can unpause anytime.                │
│                                          │
│       [Cancel]  [Pause Vault]            │
└─────────────────────────────────────────┘
```

#### Revoke Access Modal
```
┌─────────────────────────────────────────┐
│  🔒 Revoke Delegate Access               │
├─────────────────────────────────────────┤
│                                          │
│  This will:                              │
│  • Terminate current session             │
│  • Return all vault funds to you        │
│  • Deactivate the vault                 │
│                                          │
│  Estimated return: 2.5 SOL               │
│  (minus network fees: ~0.001 SOL)        │
│                                          │
│  ⚠️ This action cannot be undone         │
│                                          │
│       [Cancel]  [Revoke & Return]        │
└─────────────────────────────────────────┘
```

#### Deposit Modal
```
┌─────────────────────────────────────────┐
│  Deposit to Vault                        │
├─────────────────────────────────────────┤
│                                          │
│  Amount                                  │
│  ┌────────────────────────────────────┐ │
│  │  1.5       [SOL]         [Max]     │ │
│  └────────────────────────────────────┘ │
│                                          │
│  Available: 3.0 SOL                      │
│  Current Vault: 2.5 SOL                  │
│  After Deposit: 4.0 SOL                  │
│                                          │
│  Approved Limit: 10.0 SOL                │
│  Remaining: 6.0 SOL                      │
│                                          │
│  ✓ Within approved limit                 │
│                                          │
│       [Cancel]  [Deposit]                │
└─────────────────────────────────────────┘
```

---

## 🎨 Design System

### Color Palette
```
Primary (Solana Brand):
  - Purple: #9945FF (primary actions)
  - Green: #14F195 (success, active states)
  - Coral: #FF6B9D (warnings)

Neutrals:
  - Background: #0A0A0F (dark mode primary)
  - Surface: #1A1A24 (cards, containers)
  - Border: #2D2D3D
  - Text Primary: #FFFFFF
  - Text Secondary: #A0A0B0

Status Colors:
  - Success: #22C55E
  - Warning: #F59E0B
  - Error: #EF4444
  - Info: #3B82F6
```

### Typography
```
Font Family: 'Inter', sans-serif

Headings:
  - H1: 48px / Bold / Letter-spacing: -0.02em
  - H2: 36px / Bold / Letter-spacing: -0.01em
  - H3: 24px / Semibold
  - H4: 20px / Semibold

Body:
  - Large: 18px / Regular / Line-height: 1.6
  - Base: 16px / Regular / Line-height: 1.5
  - Small: 14px / Regular / Line-height: 1.4
  - Tiny: 12px / Medium / Line-height: 1.3

Code/Mono:
  - Font: 'JetBrains Mono', monospace
  - Used for: Wallet addresses, transaction hashes
```

### Component Styles

#### Buttons
```css
Primary Button:
  - Background: Linear gradient (#9945FF → #7B2FE8)
  - Text: White / 16px / Semibold
  - Padding: 12px 24px
  - Border-radius: 8px
  - Hover: Brightness(1.1)
  - Active: Scale(0.98)

Secondary Button:
  - Border: 2px solid #9945FF
  - Background: Transparent
  - Text: #9945FF / 16px / Semibold
  - Hover: Background #9945FF20

Danger Button:
  - Background: #EF4444
  - Text: White
  - Hover: Background #DC2626
```

#### Cards
```css
Standard Card:
  - Background: #1A1A24
  - Border: 1px solid #2D2D3D
  - Border-radius: 12px
  - Padding: 24px
  - Box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1)

Elevated Card (active/interactive):
  - Background: #1F1F2E
  - Border: 1px solid #9945FF40
  - Box-shadow: 0 8px 16px rgba(153, 69, 255, 0.1)
```

#### Input Fields
```css
Text Input:
  - Background: #0A0A0F
  - Border: 1px solid #2D2D3D
  - Border-radius: 8px
  - Padding: 12px 16px
  - Text: #FFFFFF / 16px
  - Focus: Border-color #9945FF, Box-shadow 0 0 0 3px #9945FF20

Number Input (amounts):
  - Same as text input
  - Right-aligned text
  - Optional unit label (SOL, USDC)
```

---

## 🔔 Real-Time Features

### WebSocket Events
```typescript
enum VaultEvent {
  TRADE_EXECUTED = 'trade_executed',
  DEPOSIT = 'deposit',
  WITHDRAWAL = 'withdrawal',
  SESSION_EXPIRING = 'session_expiring',
  SESSION_EXPIRED = 'session_expired',
  VAULT_PAUSED = 'vault_paused',
  ACCESS_REVOKED = 'access_revoked',
}

// Subscribe to vault events
socket.on(VaultEvent.TRADE_EXECUTED, (data) => {
  // Update trade list
  // Update balances
  // Show toast notification
});
```

### Notifications System
```
Toast Notifications:
  Position: Top-right
  Duration: 5s (auto-dismiss)
  Types:
    - Success: Green background, checkmark icon
    - Warning: Orange background, warning icon
    - Error: Red background, X icon
    - Info: Blue background, info icon

Examples:
  ✅ "Trade executed: 0.5 SOL swapped"
  ⚠️ "Session expires in 5 minutes"
  ❌ "Insufficient funds for trade"
  ℹ️ "Vault paused successfully"
```

---

## 📊 Data Visualization

### Activity Chart
```typescript
interface ActivityChartData {
  timestamp: number;
  tradeCount: number;
  volumeSOL: number;
  cumulativeSpent: number;
}

// Chart config
{
  type: 'line',
  xAxis: 'time' (hourly buckets),
  yAxis: 'SOL amount',
  series: [
    { name: 'Trade Volume', color: '#14F195' },
    { name: 'Cumulative Spent', color: '#9945FF', dashed: true }
  ]
}
```

### Session Timer
```
Visual countdown component:

┌─────────────────────┐
│   ⏱️  45:32          │
│   Minutes Remaining │
│                     │
│  [███████░░░] 75%   │  <- Progress bar
└─────────────────────┘

Colors:
  > 30 min: Green
  10-30 min: Yellow
  < 10 min: Red (pulsing)
  < 5 min: Red + notification
```

---

## 🔐 Security UI Elements

### Transaction Confirmation
```
Before ANY transaction:

┌─────────────────────────────────────────┐
│  🔐 Confirm Transaction                  │
├─────────────────────────────────────────┤
│                                          │
│  Action: Create Vault                   │
│  Network: Solana Mainnet                │
│                                          │
│  Details:                                │
│  • Approved Amount: 10 SOL              │
│  • Initial Deposit: 2.5 SOL             │
│  • Delegate: 8xKd...m9Pq                │
│                                          │
│  Estimated Fees: 0.001 SOL               │
│                                          │
│  [ ] I understand this cannot be undone │
│                                          │
│       [Cancel]  [Sign Transaction]       │
└─────────────────────────────────────────┘
```

### Security Indicators
```
Top-right corner indicators:

🟢 Secure Connection
🔒 Wallet Connected
⚠️ Testnet Mode (if applicable)
```

---

## 📱 Responsive Breakpoints

```css
/* Mobile First Approach */
xs: 0px      /* Mobile portrait */
sm: 640px    /* Mobile landscape */
md: 768px    /* Tablet portrait */
lg: 1024px   /* Tablet landscape / Desktop */
xl: 1280px   /* Desktop */
2xl: 1536px  /* Large desktop */
```

### Mobile Layout Adjustments
- Stack cards vertically
- Bottom navigation bar
- Drawer for settings
- Simplified trade table (swipe for details)
- Full-screen modals

---

## 🧩 Component Architecture

```
src/
├── components/
│   ├── common/
│   │   ├── Button.tsx
│   │   ├── Card.tsx
│   │   ├── Input.tsx
│   │   ├── Modal.tsx
│   │   ├── Toast.tsx
│   │   └── Spinner.tsx
│   ├── vault/
│   │   ├── VaultOverview.tsx
│   │   ├── SessionManager.tsx
│   │   ├── QuickActions.tsx
│   │   └── VaultStats.tsx
│   ├── trading/
│   │   ├── TradeHistory.tsx
│   │   ├── TradeRow.tsx
│   │   └── ActivityChart.tsx
│   ├── modals/
│   │   ├── DepositModal.tsx
│   │   ├── WithdrawModal.tsx
│   │   ├── PauseModal.tsx
│   │   └── RevokeModal.tsx
│   └── wallet/
│       ├── WalletButton.tsx
│       └── WalletDisplay.tsx
├── hooks/
│   ├── useVault.ts
│   ├── useSession.ts
│   ├── useTrades.ts
│   └── useWebSocket.ts
├── contexts/
│   ├── VaultContext.tsx
│   └── NotificationContext.tsx
├── utils/
│   ├── solana.ts
│   ├── formatting.ts
│   └── validation.ts
└── pages/
    ├── index.tsx (Landing)
    ├── dashboard.tsx
    └── create.tsx
```

---

## 🔄 State Management

### Context Structure
```typescript
// VaultContext
interface VaultContextType {
  vault: VaultAccount | null;
  isLoading: boolean;
  error: string | null;
  
  // Actions
  createVault: (params: CreateVaultParams) => Promise<void>;
  deposit: (amount: number) => Promise<void>;
  withdraw: (amount: number) => Promise<void>;
  pause: () => Promise<void>;
  revoke: () => Promise<void>;
  
  // Session
  approveDelegate: (delegate: string, duration?: number) => Promise<void>;
  renewSession: () => Promise<void>;
  
  // Refresh
  refresh: () => Promise<void>;
}
```

### Data Flow
```
User Action → Component → Context Hook → Program Call → 
→ Transaction → Confirmation → State Update → UI Refresh
```

---

## 🎬 Animations & Micro-interactions

### Page Transitions
- Fade in: 200ms ease-in-out
- Slide up: 300ms ease-out (modals)

### Interactive Elements
```css
Button Hover:
  - Transform: translateY(-2px)
  - Shadow: Increase intensity
  - Duration: 150ms

Card Hover (if clickable):
  - Border: Glow effect
  - Transform: scale(1.02)
  - Duration: 200ms

Loading States:
  - Skeleton screens (pulse animation)
  - Spinner for async actions
```

### Status Indicators
```
Session Timer:
  - Smooth countdown (1s intervals)
  - Color shift as expiry approaches
  - Pulse animation when < 5 min

Trade Execution:
  - Pending: Spinning loader
  - Success: Green checkmark (scale in)
  - Failed: Red X (shake animation)
```

---

## 🌐 Multi-language Support (Future)

```typescript
// i18n structure
{
  en: {
    dashboard: {
      title: "Dashboard",
      availableBalance: "Available Balance",
      // ...
    }
  },
  es: { /* Spanish */ },
  zh: { /* Chinese */ },
  // ...
}
```

---

## ♿ Accessibility

### Requirements
- WCAG 2.1 AA compliance
- Keyboard navigation support
- Screen reader optimization
- Focus indicators
- ARIA labels
- Color contrast ratios (4.5:1 minimum)

### Implementation
```tsx
<button
  aria-label="Emergency pause vault"
  aria-describedby="pause-description"
  onClick={handlePause}
>
  Pause
</button>
<span id="pause-description" className="sr-only">
  Immediately stop all trading activity
</span>
```

---

## 🧪 Testing Considerations

### E2E Test Scenarios
1. Create vault flow
2. Deposit → Trade → Withdraw cycle
3. Session expiry handling
4. Emergency pause/revoke
5. Wallet connection/disconnection

### Visual Regression
- Screenshot tests for all components
- Cross-browser testing (Chrome, Firefox, Safari)
- Mobile device testing (iOS, Android)

---

## 📈 Analytics Events

```typescript
// Track user actions
analytics.track('vault_created', {
  approvedAmount: 10,
  initialDeposit: 2.5,
  sessionDuration: 3600
});

analytics.track('trade_executed', {
  amount: 0.5,
  fee: 0.001,
  tradeNumber: 23
});

analytics.track('session_renewed', {
  timeRemaining: 300
});

// Performance metrics
analytics.track('page_load', {
  page: 'dashboard',
  loadTime: 1200
});
```

---

## 🚀 Performance Optimization

### Code Splitting
```typescript
// Lazy load heavy components
const ActivityChart = lazy(() => import('./ActivityChart'));
const TradeHistory = lazy(() => import('./TradeHistory'));
```

### Caching Strategy
```typescript
// Cache vault data
const { data, isStale } = useQuery({
  queryKey: ['vault', vaultPda],
  queryFn: fetchVaultData,
  staleTime: 5000, // 5 seconds
  cacheTime: 300000, // 5 minutes
});
```

### Image Optimization
- WebP format with fallbacks
- Lazy loading for below-fold images
- Optimized SVG icons

---

## 🎯 Key UX Principles

1. **Clarity Over Cleverness**
   - Clear labels, no jargon
   - Explicit confirmation for destructive actions

2. **Progressive Disclosure**
   - Show essential info first
   - Advanced options behind "Advanced" toggle

3. **Immediate Feedback**
   - Loading states for all async actions
   - Toast notifications for confirmations
   - Real-time updates via WebSocket

4. **Error Recovery**
   - Helpful error messages
   - Suggest solutions
   - Allow retry without losing data

5. **Mobile-First**
   - Touch-friendly targets (min 44px)
   - One-handed operation where possible
   - Simplified navigation

---

## 📋 Component Checklist

### Must-Have Components
- [x] Wallet connection button
- [x] Vault overview card
- [x] Session manager with countdown
- [x] Trade history table
- [x] Deposit/Withdraw forms
- [x] Emergency controls
- [x] Activity chart
- [x] Transaction confirmation modals
- [x] Toast notification system
- [x] Loading states & skeletons

### Nice-to-Have Components
- [ ] Dark/Light mode toggle
- [ ] Export trade history (CSV)
- [ ] Notification preferences
- [ ] Multi-vault management
- [ ] Delegate address book
- [ ] Tutorial/onboarding flow
- [ ] Performance analytics dashboard

---

## 🔗 External Integrations

### Required
- **Wallet Adapters**: Phantom, Solflare, Ledger
- **RPC Provider**: Helius, QuickNode, or Alchemy
- **DEX Aggregator**: Jupiter API (for actual trades)

### Optional
- **Price Feeds**: CoinGecko API, Pyth Network
- **Analytics**: Mixpanel, Amplitude
- **Monitoring**: Sentry for error tracking
- **Notifications**: Push notifications (OneSignal)

---

This design map provides a comprehensive blueprint for building the Ephemeral Vault frontend. The design prioritizes security, clarity, and real-time feedback while maintaining a modern, Solana-native aesthetic.
