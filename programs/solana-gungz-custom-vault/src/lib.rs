use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

declare_id!("6SWg9b4teAfSHaYFbarF4KBAXnfyhBcwUkveP3N324TL");

#[program]
pub mod solana_gungz_custom_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }

    pub fn remove_member(ctx: Context<RemoveMember>) -> Result<()> {
        ctx.accounts.remove_member()
    }

    pub fn add_member(ctx: Context<AddMember>, amount: u64, unlock_ts: i64) -> Result<()> {
        ctx.accounts.add_member(&ctx.bumps, amount, unlock_ts)
    }

    pub fn member_withdraw(ctx: Context<MemberWithdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init, payer = user, seeds = [b"state", user.key().as_ref()], bump, space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE)]
    pub vault_state: Account<'info, VaultState>,
    #[account(mut, seeds = [b"vault", vault_state.key().as_ref()], bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bump: &InitializeBumps) -> Result<()> {
        let rent_exempt = Rent::get()?.minimum_balance(self.vault.to_account_info().data_len());
        let cpi_programs = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_programs, cpi_accounts);
        transfer(cpi_ctx, rent_exempt)?;

        self.vault_state.vault_bump = bump.vault;
        self.vault_state.state_bump = bump.vault_state;
        self.vault_state.owner = self.user.key();
        self.vault_state.member_count = 0;
        Ok(())   
    }   
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [b"state", user.key().as_ref()], 
        bump = vault_state.state_bump,
        constraint = vault_state.owner == user.key() @ VaultError::Unauthorized
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        mut, 
        seeds = [b"vault", vault_state.key().as_ref()], 
        bump = vault_state.vault_bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_programs = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_programs, cpi_accounts);
        transfer(cpi_ctx, amount)?;

        Ok(())   
    }   
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [b"state", user.key().as_ref()], 
        bump = vault_state.state_bump,
        constraint = vault_state.owner == user.key() @ VaultError::Unauthorized
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        mut, 
        seeds = [b"vault", vault_state.key().as_ref()], 
        bump = vault_state.vault_bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
   pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let cpi_programs = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", self.vault_state.to_account_info().key.as_ref(), &[self.vault_state.vault_bump]]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_programs, cpi_accounts, signer_seeds);
        transfer(cpi_ctx, amount)?;
        Ok(())    
    }
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        close = user,
        seeds = [b"state", user.key().as_ref()], 
        bump = vault_state.state_bump,
        constraint = vault_state.owner == user.key() @ VaultError::Unauthorized,
        constraint = vault_state.member_count == 0 @ VaultError::MembersExist
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        mut, 
        seeds = [b"vault", vault_state.key().as_ref()], 
        bump = vault_state.vault_bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Close<'info> {
    pub fn close(&mut self) -> Result<()> {
        let cpi_programs = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", self.vault_state.to_account_info().key.as_ref(), &[self.vault_state.vault_bump]]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_programs, cpi_accounts, signer_seeds);
        transfer(cpi_ctx, self.vault.to_account_info().lamports())?;
        Ok(())    
    }
}

#[derive(Accounts)]
pub struct MemberWithdraw<'info> {
    #[account( 
        mut,
        seeds = [b"member_state", vault_state.key().as_ref(), member.key().as_ref()],
        bump = member_account.state_bump,
        has_one = vault_state
    )]
    pub member_account: Account<'info, MemberAccount>,
    #[account(
        seeds = [b"state", vault_state.owner.as_ref()], 
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(mut)]
    pub member: Signer<'info>,
    #[account(
        mut, 
        seeds = [b"vault", vault_state.key().as_ref()], 
        bump = vault_state.vault_bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> MemberWithdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let current_ts = Clock::get()?.unix_timestamp as i64;
        require!(current_ts >= self.member_account.unlock_ts, VaultError::StillLocked);
        require!(amount <= self.vault.to_account_info().lamports(), VaultError::LimitMustBeLowerThanVaultBalance);
        require!(self.member_account.amount_withdrawn + amount <= self.member_account.limit_amount, VaultError::ExceedsLimit);

        if self.member_account.last_withdrawal_ts > 0 {
            require!(current_ts - self.member_account.last_withdrawal_ts >= self.member_account.reset_interval, VaultError::WithdrawalLimit);    
        }
        self.member_account.amount_withdrawn += amount;
        self.member_account.last_withdrawal_ts = current_ts;

        let cpi_programs = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.member.to_account_info(),
        };

        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", self.vault_state.to_account_info().key.as_ref(), &[self.vault_state.vault_bump]]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_programs, cpi_accounts, signer_seeds);
        transfer(cpi_ctx, amount)?;
        
        Ok(())   
    }   
}

#[derive(Accounts)]
pub struct RemoveMember<'info> {
    #[account(
        mut,
        has_one = owner,
        constraint = vault_state.member_count > 0 @ VaultError::MemberCountUnderflow
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        mut,
        close = owner,
        seeds = [b"member_state", vault_state.key().as_ref(), user_to_remove.key().as_ref()],
        bump = member_account.state_bump
    )]
    pub member_account: Account<'info, MemberAccount>,
    pub user_to_remove: SystemAccount<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

impl<'info> RemoveMember<'info> {
    pub fn remove_member(&mut self) -> Result<()> {
        self.vault_state.member_count -= 1;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct AddMember<'info> {
    #[account(mut, has_one = owner)]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        init, 
        payer = owner, 
        space = MemberAccount::DISCRIMINATOR.len() + MemberAccount::INIT_SPACE,
        seeds = [b"member_state", vault_state.key().as_ref(), user_to_add.key().as_ref()],
        bump
    )]
    pub member_account: Account<'info, MemberAccount>,
    pub user_to_add: SystemAccount<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut, 
        seeds = [b"vault", vault_state.key().as_ref()], 
        bump = vault_state.vault_bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> AddMember<'info> {
    pub fn add_member(&mut self, bump: &AddMemberBumps, amount: u64, unlock_ts: i64) -> Result<()> {
        
        let current_ts = Clock::get()?.unix_timestamp as i64;
        require!(unlock_ts > current_ts, VaultError::WithdrawalMustBeFuture);
        require!(amount <= self.vault.to_account_info().lamports(), VaultError::LimitMustBeLowerThanVaultBalance);

        self.member_account.owner = self.owner.key();
        self.member_account.user = self.user_to_add.key();
        self.member_account.vault_state = self.vault_state.key();
        self.member_account.state_bump = bump.member_account;
        self.member_account.limit_amount = amount;
        self.member_account.reset_interval = 24 * 3600; // 24 hours
        self.member_account.last_withdrawal_ts = 0;
        self.member_account.amount_withdrawn = 0;
        self.member_account.unlock_ts = unlock_ts;
        self.vault_state.member_count += 1;
        Ok(())   
    }   
}

#[derive(InitSpace)]
#[account]
pub struct VaultState {
    pub owner: Pubkey,
    pub vault_bump: u8,
    pub state_bump: u8,
    pub member_count: u32,
}

#[derive(InitSpace)]
#[account]
pub struct MemberAccount {
    pub owner: Pubkey,
    pub vault_state: Pubkey,
    pub user: Pubkey,
    pub state_bump: u8,
    pub limit_amount: u64,
    pub reset_interval: i64,
    pub last_withdrawal_ts: i64,
    pub amount_withdrawn: u64,
    pub unlock_ts: i64,
}

#[error_code]
pub enum VaultError {
    #[msg("Withdrawal must be in future")] WithdrawalMustBeFuture,
    #[msg("Limit must be lower than vault balance")] LimitMustBeLowerThanVaultBalance,
    #[msg("Vault locked")] StillLocked,
    #[msg("Limit exceeded")] ExceedsLimit,
    #[msg("Cannot close vault with existing members")] MembersExist,
    #[msg("Member count underflow")] MemberCountUnderflow,
    #[msg("Only can withdraw once per day")] WithdrawalLimit,
    #[msg("Unauthorized")] Unauthorized,
}