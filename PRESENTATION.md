# Solana Custom Vault - 2 Minute Presentation


## 🏗️ Vault Design

### **Core Features**
Owner Operations:
1. Initialize vault
2. Deposit/Withdraw (full control)
3. Add members (with limits & unlock time)
4. Remove members
5. Close vault (when no member)

Member Operations:
1. Withdraw (within limits, after unlock time)

### **Security Controls**
- ✅ **Time-locks**: Members can't withdraw before unlock_ts
- ✅ **Rate limiting**: Only one withdrawal per certain time range (24h)
- ✅ **Amount caps**: Total withdrawal amount (caps) per member
- ✅ **Owner authorization**: Only owner can manage vault
- ✅ **Member tracking**: Prevents closing vault with active members

### **Smart Constraints**
- Members can only withdraw after unlock time
- Withdrawal reset every 24 hours
- Owner verified on all critical operations
- Vault can't close with existing members

---

## 🎯 Use Cases

### 1. **Employee Salary Vault**
- Company deposits monthly payroll into vault
- Employees added as members with salary as withdrawal limits
- Prevents overspending while ensuring access to earned wages
- Employer controls member access

### 2. **Competition Prize Distribution**
- Contest organizer locks prize pool in vault
- Winners added as members with time-locked access
- Each winner has withdrawal limit (prevents draining entire pool)
- Organizer controls member access

### 3. **Allowance/Stipend System**
- Parents/Organizations fund vault
- Beneficiaries get spending limits
- Time-locked until specific date (e.g., college semester start)
- Automatic reset intervals for recurring allowances

### 4. **Escrow with Rate Limiting**
- Funds locked until unlock timestamp
- Gradual release through daily limits
- Owner maintains control to add/remove beneficiaries

---

## 🧪 Test Scenarios

### **Happy Path**
1. ✅ Initialize vault → Owner deposits 1 SOL
2. ✅ Add member (0.1 SOL limit, 10s unlock)
3. ✅ Member withdraws 0.05 SOL after unlock
4. ✅ Remove member → Close vault

### **Security Tests**
1. ❌ **Member withdraw before unlock** → `StillLocked` error
2. ❌ **Member exceeds limit** → `ExceedsLimit` error  
   - First withdrawal: 0.05 SOL ✅
   - Second withdrawal: 0.1 SOL ❌ (total would be 0.15)
3. ❌ **Close vault with members** → `MembersExist` error

### **Edge Cases**
- Owner-only operations (deposit/withdraw/close)
- Member count tracking (increment/decrement)
- Automatic limit reset after 24 hours
- Rent-exempt vault initialization

---

## 📊 Key Metrics

| Feature | Implementation |
|---------|---------------|
| **Accounts** | VaultState (owner data) + MemberAccount (per member) |
| **PDAs** | 3 types: state, vault, member_state |
| **Functions** | 7 instructions (init, deposit, withdraw, close, add/remove member, member_withdraw) |
| **Security** | 7 custom errors + constraint-based validation |

---

## 🚀 Demo Flow (30 seconds)

```bash
1. anchor test
   ├─ Initialize vault ✓
   ├─ Deposit 1 SOL ✓
   ├─ Add member (locked) ✓
   ├─ Member withdraw (fails - locked) ✓
   ├─ Wait 11s...
   ├─ Member withdraw 0.05 SOL ✓
   ├─ Member withdraw 0.1 SOL (fails - limit) ✓
   ├─ Remove member ✓
   └─ Close vault ✓
```
Program ID: 6SWg9b4teAfSHaYFbarF4KBAXnfyhBcwUkveP3N324TL (Devnet)

## 💡 Innovation Highlights
1. Dual Control Model: Owner has full control, members have restricted access

2. Time + Amount Limits: Combines time-locks with rate limiting

3. Auto-Reset Mechanism: Daily withdrawal reset automatically (no manual intervention)

4. Secure by Design: Constraint-based validation, no UncheckedAccount

5. Real-world Ready: Solves actual problems