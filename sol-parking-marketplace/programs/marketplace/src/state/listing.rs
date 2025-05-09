use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub maker: Pubkey,
    pub bump: u8, 
    #[max_len(32)] 
    pub address: String,
    pub rental_rate: u64, //per hour
    pub sensor_id: u64, 
    pub reserved_by: Option<Pubkey>, 
    pub reservation_duration: Option<u64>,
    pub parking_space_status:ParkingSpaceStatus, 
}

#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum ParkingSpaceStatus {
    Available,
    Reserved,
    Occupied,
    UnAvailable
}

// Implement the Space trait for ParkingSpaceStatus
impl Space for ParkingSpaceStatus {
    //const LEN: usize = 1;
                                    // rust-analyer popup mentioned 42, trying 42
    // const INIT_SPACE: usize = 42; // Since we are using u8 representation
    const INIT_SPACE: usize = 1; 

}