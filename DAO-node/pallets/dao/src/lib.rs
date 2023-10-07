#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

type DaoId = u16;
type KeccakHash = [u8; 32];
type EntropyHash = [u8; 32];

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Randomness;
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::vec::Vec;
	use scale_info::prelude::collections::BTreeSet;
	use sp_runtime::traits::Saturating;
	use sp_io::hashing::keccak_256;


	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
		type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
		/// The minimum length a dao name.
		#[pallet::constant]
		type MinNameLength: Get<u32>;
		/// The maximum length of a dao name.
		#[pallet::constant]
		type MaxNameLength: Get<u32>;
		/// The maximum length of a dao name.
		#[pallet::constant]
		type MaxMembersLength: Get<u32>;
		/// The maximum length of a dao name.
		#[pallet::constant]
		type DaoNumberGenerator: Get<u8>;
	}

	#[pallet::storage]
	pub type DaoIdCounter<T: Config> = StorageValue<_, DaoId, ValueQuery>;

	#[pallet::storage]
	pub type DAOIdMap<T: Config> =
		StorageMap<_, Blake2_128Concat, DaoId, BoundedVec<u8, T::MaxNameLength>, OptionQuery>;

	#[pallet::storage]
	pub type DAOIdStorage<T: Config> =
		StorageMap<_, Blake2_128Concat, BoundedVec<u8, T::MaxNameLength>, DaoId, OptionQuery>;

	#[pallet::storage]
	pub type DAOOwner<T: Config> =
		StorageMap<_, Blake2_128Concat, DaoId, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type DAOMembers<T: Config> =
		StorageMap<_, Blake2_128Concat, DaoId, BoundedVec<T::AccountId, T::MaxNameLength>, ValueQuery>;

	#[pallet::storage]
	pub type DaoRoundStartBlock<T: Config> =
		StorageMap<_, Blake2_128Concat, DaoId, BlockNumberFor<T>, OptionQuery>;

	#[pallet::storage]
	pub type RandomNumberParticipant<T: Config> =
			StorageDoubleMap<_, Blake2_128Concat, DaoId, Blake2_128Concat, T::AccountId, (EntropyHash, KeccakHash), OptionQuery>;

	#[pallet::storage]
	pub type WinningHashes<T: Config> =
			StorageMap<_, Blake2_128Concat, DaoId, KeccakHash, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DAOCreated { owner: T::AccountId, id: DaoId, name: BoundedVec<u8, T::MaxNameLength> },
		MemberInserted { id: DaoId, member_id: T::AccountId },
		MemberRemoved { id: DaoId, member_id: T::AccountId },
		MaskedValueReceived { id: DaoId, member: T::AccountId },
		ActualValueReceived { id: DaoId, member: T::AccountId, hash_bytes: KeccakHash, value: u64 }
 	}

	#[pallet::error]
	pub enum Error<T> {
		//Dao name too long
		TooLong,
		//Dao name too short
		TooShort,
		//Dao already exists
		DAOExists,
		//Dao does not exists
		DAODoestNotExists,
		//Dao does not exists
		InvalidOwner,
		//Member does not exists
		MemberDoesNotExists,
		//Member already exsits
		MemberAlreadyExists,
		//Member already exsits
		MemberLengthReachedMax,
		//Member already exsits
		InvalidHashProvided,
		//Member already exsits
		NoWinningHash(DaoId),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn create_dao(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let bounded_name: BoundedVec<_, _> =
				name.try_into().map_err(|_| Error::<T>::TooLong)?;
			ensure!(bounded_name.len() >= T::MinNameLength::get() as usize, Error::<T>::TooShort);
			ensure!(<DAOIdStorage<T>>::get(bounded_name.clone()).is_none(), Error::<T>::DAOExists);

			let dao_id = <DaoIdCounter<T>>::get();
			<DAOIdMap<T>>::insert(dao_id, bounded_name.clone());
			<DAOIdStorage<T>>::insert(bounded_name.clone(), dao_id);
			<DAOOwner<T>>::insert(dao_id, who.clone());
			<DaoIdCounter<T>>::put(dao_id.saturating_plus_one());
			Self::deposit_event(Event::DAOCreated { owner: who, id: dao_id, name: bounded_name });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn add_member(origin: OriginFor<T>, dao_id: DaoId, account_id: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(<DAOIdMap<T>>::get(dao_id).is_some(), Error::<T>::DAODoestNotExists);
			ensure!(<DAOOwner<T>>::get(dao_id) == Some(who), Error::<T>::InvalidOwner);
			let mut members = <DAOMembers<T>>::get(&dao_id);
			ensure!(!members.contains(&account_id.clone()), Error::<T>::MemberAlreadyExists);
			members.try_push(account_id.clone()).map_err(|_| Error::<T>::MemberLengthReachedMax)?;
			<DAOMembers<T>>::insert(dao_id, members);
			Self::deposit_event(Event::MemberInserted {
				id: dao_id,
				member_id: account_id
			});
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn remove_member(origin: OriginFor<T>, dao_id: DaoId, account_id: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(<DAOIdMap<T>>::get(dao_id).is_some(), Error::<T>::DAODoestNotExists);
			ensure!(<DAOOwner<T>>::get(dao_id) == Some(who), Error::<T>::InvalidOwner);
			let mut members = <DAOMembers<T>>::get(&dao_id);
			ensure!(members.contains(&account_id.clone()), Error::<T>::MemberDoesNotExists);
			members.retain(|member| *member != account_id);
			<DAOMembers<T>>::insert(dao_id, members);
			Self::deposit_event(Event::MemberInserted {
				id: dao_id,
				member_id: account_id
			});
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn submit_masked_value(origin: OriginFor<T>, dao_id: DaoId, entropy: EntropyHash, hash_bytes: KeccakHash) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let members = <DAOMembers<T>>::get(&dao_id);
			ensure!(members.contains(&who), Error::<T>::MemberDoesNotExists);
			<RandomNumberParticipant<T>>::insert(dao_id, who.clone(), (entropy, hash_bytes));
			if <DaoRoundStartBlock<T>>::get(dao_id).is_none() {
				<DaoRoundStartBlock<T>>::insert(dao_id, frame_system::Pallet::<T>::block_number())
			}
			Self::deposit_event(Event::MaskedValueReceived {
				id: dao_id,
				member: who
			});
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn reveal_value(origin: OriginFor<T>, dao_id: DaoId, hash_bytes: KeccakHash, value: u64) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let members = <DAOMembers<T>>::get(&dao_id);
			ensure!(members.contains(&who), Error::<T>::MemberDoesNotExists);
			let bytes: [u8; 8] = value.to_le_bytes();
			let verify_hash = keccak_256(&bytes);
			ensure!(verify_hash == hash_bytes, Error::<T>::InvalidHashProvided);
			let hash = <WinningHashes<T>>::get(dao_id).ok_or(Error::<T>::NoWinningHash(dao_id))?;
			if hash == verify_hash{
				Self::deposit_event(Event::ActualValueReceived {
					id: dao_id,
					member: who,
					hash_bytes,
					value,
				});
			}
			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {
			<DaoRoundStartBlock<T>>::iter().for_each(|(dao_id, block)| {
				if n < (block + T::DaoNumberGenerator::get().into()) {
					Self::generate_random_number(dao_id);
				}
			})
		}
	}

	impl<T: Config> Pallet<T> {
		fn generate_random_number(id: DaoId) {
			let on_chain_random_seed = T::Randomness::random_seed();
			let mut entropies = BTreeSet::new();
			let mut hashes: Vec<KeccakHash> = Vec::new();
			<RandomNumberParticipant<T>>::iter_prefix(id).for_each(|(_, (entropy, hash))| {
				entropies.insert(entropy);
				hashes.push(hash);
			});
			let all_entropies_bytes: Vec<u8> = entropies.iter().fold(Vec::new(), |mut acc, &item|{
				acc.extend_from_slice(&item);
				acc
			});
			let combined_entropy = &[&on_chain_random_seed.encode()[..], &all_entropies_bytes[..]].concat();
			let hash = keccak_256(combined_entropy);
			let index = (hash.as_ref()[0] as usize) % hashes.len();
			let selected_choice = hashes[index];
			<WinningHashes<T>>::insert(id, selected_choice);
		}
	}
	
}
