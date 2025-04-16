use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Marketplace {
    pub admin: Pubkey,
    pub fee: u16, //basis points
    pub bump: u8, 
   // pub treasury: Pubkey,
    pub treasury_bump: u8, 
    pub rewards_bump: u8,
    #[max_len(32)] 
    pub name: String

}