#![allow(unexpected_cfgs)]
use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

//https://gist.github.com/bergabman/3387abe5bcaf8a9e86aeee24577b5719

//anchor keys sync
declare_id!("3YJsLgDvMoRHr5ttc19ZVdvTVfFHWE81FVcBgWLBKTFb");//function-like macro

#[program]//attribute-like macro, can insert or remove fields
pub mod vault {//instructions go here


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

}

#[derive(Accounts)] //custom derive macro, - augments struct,does not alter  , adds impl, ex #[derive(Debug)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,      //signs transaction, mut pays for creation of state account
    #[account(
        seeds = [b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,        //PDA, will hold funds, SystemAccount owned by SystemProgram and this program will be signer fo vault
                                            //vault is only derived, no changes made yet, so mut is not needed
                                            //will be used to transfer SOL in and out
                                            //initialization will be when you send SOL to it
                                            //no data, so init not needed
    #[account(
        init,                               //init implies mut
        payer = signer,
        seeds = [b"state", signer.key().as_ref()],
        bump,
        space = 8 + VaultState::INIT_SPACE, //Anchor discriminator, 8 byte  string lets anchor find account
    )]
    pub vault_state: Account<'info, VaultState>,    //PDA, stores bumps, 'our' Account, it uses Account struct

    pub system_program: Program<'info, System>
}

impl<'info> Initialize<'info> { //storing bumps saves cost, CU
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.vault_bump = bumps.vault;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,//vault balance will increase due to deposit, so must be mutable

    #[account(
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {

        //when depositing from Signer, need to ask System program, so we need to setup system program context
        let cpi_program = self.system_program.to_account_info();

        //then we need accounts
        let cpi_account = Transfer {
            from: self.signer.to_account_info(),
            to: self.vault.to_account_info()
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_account);
        //when Deposit ix called, it is signed and the signature carries through
        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>, //this is here for vault acct

    pub system_program: Program<'info, System>
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {

        let cpi_program = self.system_program.to_account_info();

        let cpi_account = Transfer {
            from: self.vault.to_account_info(),
            to: self.signer.to_account_info()
        };

        //from withdraw::vault account seeds -         seeds = [b"vault", vault_state.key().as_ref()], 
        let pda_signing_seeds = [
            b"vault", 
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump]
        ];
        let signer_seeds = &[&pda_signing_seeds[..]];
        //new_with_signer since we are sending SOL back to signer
        //need to setup seeds to sign with the seeds
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, signer_seeds);

        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut, 
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump,
        close = signer //who will receive funds upon closing, zeroes out data
    )]
    pub vault_state: Account<'info, VaultState>, //this is here for vault acct

    pub system_program: Program<'info, System>
}

impl<'info> Close<'info> {
    pub fn close(&mut self) -> Result<()> {
        //close vault_state and vault

        let cpi_program = self.system_program.to_account_info();

        let cpi_account = Transfer {
            from: self.vault.to_account_info(),
            to: self.signer.to_account_info()
        };

        //from withdraw::vault account seeds -         seeds = [b"vault", vault_state.key().as_ref()], (line 96,97)
        let pda_signing_seeds = [
            b"vault", 
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump]
        ];
        let signer_seeds = &[&pda_signing_seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, signer_seeds);

        transfer(cpi_ctx, self.vault.lamports())

    }
}


#[account]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}

impl Space for VaultState {
    const INIT_SPACE: usize = 1 + 1; //vault_bump is 1 byte, state_bump 1 byte
}
