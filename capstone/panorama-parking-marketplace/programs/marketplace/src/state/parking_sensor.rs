use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ParkingSensor {
    pub triggered: bool,
  
}