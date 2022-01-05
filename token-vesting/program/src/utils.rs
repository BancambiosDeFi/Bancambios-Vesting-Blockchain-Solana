use borsh::BorshSerialize;
use solana_program::{account_info::AccountInfo, entrypoint_deprecated::ProgramResult};

pub fn write_to_storage<T>(data: T, account: &AccountInfo) -> ProgramResult
where
    T: BorshSerialize,
{
    let bytes = data.try_to_vec()?;
    let mut storage = account.try_borrow_mut_data()?;
    storage[0..bytes.len()].clone_from_slice(&bytes);

    Ok(())
}
