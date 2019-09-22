/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references


/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs

use support::{decl_storage, decl_module, StorageValue, StorageMap,
            dispatch::Result, ensure, decl_event, traits::{Currency, ReservableCurrency}};
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash, Zero};
use rstd::prelude::*;
extern crate rand;
use rand::Rng;
use rand::prelude::*;
use parity_codec::{Encode, Decode};

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Lottery<Hash, AccountId, Balance, BlockNumber>{
    // the  id of a lottery
    id: Hash,
    // lottery owner
    owner: AccountId,
    // lottery name 
    name: Vec<u8>,
    // bonus to send 
    bonus: Balance,

	// min invest amount
	min_invest: Balance,
    //  lottery deadline
    expiry: BlockNumber,
    // status 0- In-process 1- Success 2- Failure
    status: u64,
}
const MAX_LOTTERY_PER_BLOCK: usize = 3;

pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// /// The module's configuration trait.
// pub trait Trait: system::Trait {
// 	// TODO: Add other types and constants required configure this module.

// 	/// The overarching event type.
// 	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
// }

decl_event!(
	pub enum Event<T> 
	where 
		<T as system::Trait>::AccountId,
		<T as system::Trait>::Hash,
		<T as balances::Trait>::Balance,
		<T as system::Trait>::BlockNumber
	{
		// Just a dummy event.
		// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
		// To emit this event, we call the deposit funtion, from our runtime funtions
		SomethingStored(u32, AccountId),

		CreateLottery(AccountId, Hash, Balance, Balance, BlockNumber),
        InvestLottery (Hash, AccountId, Balance),
        FinalizeLottery(Hash, Balance, BlockNumber, bool),
	}
);

/// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as luckyplus {
		// Just a dummy storage item. 
		// Here we are declaring a StorageValue, `Something` as a Option<u32>
		// `get(something)` is the default getter which returns either the stored `u32` or `None` if nothing stored
		Something get(something): Option<u32>;

		// Global state
        Lotterys get(lottery_by_id): map T::Hash => Lottery<T::Hash, T::AccountId, T::Balance, T::BlockNumber>;

		// Owner of a Lottery
        LotteryOwner get(owner_of): map T::Hash => Option<T::AccountId>;

		// All Lottery state
        AllLotteryArray get(lottery_by_index): map u64 => T::Hash;
        AllLotteryCount get(all_lottery_count): u64;
        AllLotteryIndex: map T::Hash => u64;

		// The owner lottery state
        OwnedLotteryArray get(lottery_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
        OwnedLotteryCount get(owned_lottery_count): map T::AccountId => u64;
        OwnedLotteryIndex: map (T::AccountId, T::Hash) => u64;

		 // The investor invested how much money for the specified lottery
        InvestAmount get(invest_amount_of): map (T::Hash, T::AccountId) => T::Balance;
        // investor that who had join the lottery 
        InvestAccounts get(invest_accounts): map T::Hash => Vec<T::AccountId>;
        InvestAccountsCount get(invest_accounts_count): map T::Hash => u64; 

        InvestedLotterysArray get(invested_lottery_by_index): map (T::AccountId, u64) => T::Hash;
        InvestedLotterysCount get(invested_lottery_count): map T::AccountId => u64;
        InvestedLotterysIndex: map (T::AccountId, T::Hash) => u64;

		LotteryInvestedAmount get(total_amount_of_invested): map T::Hash => T::Balance;	
		// Lottery ending in a block
        LotterysByBlockNumber get(lottery_expire_at): map T::BlockNumber => Vec<T::Hash>;
		//  the count of lottery
        Nonce: u64;


	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

		// Just a dummy entry point.
		// function that can be called by the external world as an extrinsics call
		// takes a parameter of the type `AccountId`, stores it and emits an event
		pub fn do_something(origin, something: u32) -> Result {
			// TODO: You only need this if you want to check it was signed.
			let who = ensure_signed(origin)?;

			// TODO: Code to execute when something calls this.
			// For example: the following line stores the passed in u32 in the storage
			<Something<T>>::put(something);

			// here we are raising the Something event
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}


		 /// Create a lottery
        fn create_lottery(origin, name: Vec<u8>, bonus: T::Balance, min_invest: T::Balance, expiry: T::BlockNumber) -> Result {
            // get the sender
            let sender = ensure_signed(origin)?;
            // get the nonce to help generate unique id
            let nonce = <Nonce<T>>::get();
            // generate the unique id
            let lottery_id = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);
            // ensure that the lottery id is unique
            ensure!(!<LotteryOwner<T>>::exists(&lottery_id), "Lottery already exists");
        
            // create a new lottery
            let new_lottery = Lottery{
                id: lottery_id.clone(),
                owner: sender.clone(),
                name: name,
                bonus: bonus,
				min_invest: min_invest,
                expiry: expiry,
                status: 0,
            };
            // ensure that the expiry is valid
            ensure!(expiry > <system::Module<T>>::block_number(), "The expiry has to be greater than the current block number");
          

            // ensure that the number of lotterys in the block does not exceed maximum
            let lotterys = Self::lottery_expire_at(expiry);
            ensure!(lotterys.len() < MAX_LOTTERY_PER_BLOCK, "Maximum number of lotterys is reached for the target block, try another block");

            Self::mint(sender.clone(), lottery_id.clone(), expiry.clone(), bonus.clone(), new_lottery)?;

            // deposit the event
            Self::deposit_event(RawEvent::CreateLottery(sender, lottery_id, bonus, min_invest, expiry));
            Ok(())
        }


		/// invest a lottery
        fn invest_lottery(origin, lottery_id: T::Hash, invest_amount: T::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            let owner = Self::owner_of(lottery_id).ok_or("No owner of the lottery")?;
            ensure!(owner != sender, "You can't invest for your own lottery");
			// add investor 
            Self::invest_process(sender.clone(), funding_id.clone(), invest_amount.clone())?;
            // deposit the event
            Self::deposit_event(RawEvent::InvestLottery(lottery_id, sender, invest_amount));

            Ok(())
        }


		fn finalize_lottery() {
            // get all the lottery of the block
            let block_number = <system::Module<T>>::block_number();
            let lottery_hashs = Self::lottery_expire_at(block_number); // todo


            for lottery_id in &lottery_hashs {
                // Get the lottery
                let mut lottery = Self:: lottery_by_id(lottery_id);
                // Get the amount of invested  money 
                let amount_of_invested = Self::total_amount_of_invested(lottery_id);

				lottery.status = 1;
				let _owner = Self::owner_of(lottery_id);
                <Lotterys<T>>::insert(lottery_id.clone(), lottery);
				// Get all the investors
                let investors = Self::invest_accounts(lottery_id);
				// Choose one randomly  
                let invested_count = Self::get_invested_number(lottery_id);
                let mut rng = rand::thread_rng();
                let invested_index = rng.gen_range(0, invested_count);

				// tranform 

				// post event
				Self::deposit_event(RawEvent::FinalizeLottery(*lottery_id, amount_of_invested, block_number, true));
                
            }
        }
	}
}



impl<T: Trait> Module<T> {

    fn mint(sender: T::AccountId, lottery_id: T::Hash, expiry: T::BlockNumber, bonus: T::Balance, new_lottery: Lottery<T::Hash, T::AccountId, T::Balance, T::BlockNumber>) -> Result{

        let all_lottery_count = Self::all_lottery_count();
        let new_all_lottery_count = all_lottery_count.checked_add(1).ok_or("Overflow adding a new lottery to total lotterys")?;

        let owned_lottery_count = Self::owned_lottery_count(&sender);
        let new_owned_funding_count = owned_lottery_count.checked_add(1).ok_or("Overflow adding a new lottery to account balance")?;

        // change the global states
        <Lotterys<T>>::insert(lottery_id.clone(), new_lottery.clone());
        <LotteryOwner<T>>::insert(lottery_id.clone(), sender.clone());

        <LotterysByBlockNumber<T>>::mutate(expiry.clone(), |lotterys| lotterys.push(lottery_id.clone()));

        
        <AllLotteryArray<T>>::insert(&all_lottery_count, lottery_id.clone());
        <AllLotteryCount<T>>::put(new_all_lottery_count);
        <AllLotteryIndex<T>>::insert(lottery_id.clone(), all_lottery_count);

    
        <OwnedLotteryArray<T>>::insert((sender.clone(), owned_lottery_count.clone()), lottery_id.clone());
        <OwnedLotteryCount<T>>::insert(&sender, new_owned_funding_count);
        <OwnedLotteryIndex<T>>::insert((sender.clone(), lottery_id.clone()), owned_lottery_count);

        if bonus > T::Balance::sa(0) {
            match Self::not_invest_before(sender.clone(), lottery_id.clone(), bonus.clone()){
                // If the invest function meets error then revert the storage
                Err(_e) => {
                    <Lotterys<T>>::remove(lottery_id.clone());
                    <LotteryOwner<T>>::remove(lottery_id.clone());
                    <LotterysByBlockNumber<T>>::mutate(expiry,|lotterys| lotterys.pop());
                    <AllLotteryArray<T>>::remove(&all_lottery_count);
                    <AllLotteryCount<T>>::put(all_lottery_count.clone());
                    <AllLotteryIndex<T>>::remove(lottery_id.clone());
                    <OwnedLotteryArray<T>>::remove((sender.clone(), owned_lottery_count.clone()));
                    <OwnedLotteryCount<T>>::remove(&sender);
                    <OwnedLotteryIndex<T>>::remove((sender.clone(), lottery_id.clone()));
                },
                Ok(_v) => {}
            }
        }

        <Nonce<T>>::mutate(|n| *n += 1);

        Ok(())
    }


  


    fn invest_process(sender: T::AccountId, lottery_id: T::Hash, invest_amount: T::Balance) -> Result{
        // ensure the lottery exists
        ensure!(<Lotterys<T>>::exists(lottery_id), "The lottery does not exist");
        // ensure that the investor has enough money
        ensure!(<balances::Module<T>>::free_balance(sender.clone()) >= invest_amount, "You don't have enough free balance for investing for the lottery");

        // get the number of projects that the investor had invested and add it
        let invested_lottery_count = Self::invested_lottery_count(&sender);
        let new_invested_lottery_count = invested_lottery_count.checked_add(1).ok_or("Overflow adding a new invested lottery")?;

        let investor_count = <InvestAccountsCount<T>>::get(&lottery_id);
        let new_investor_count = investor_count.checked_add(1).ok_or("Overflow adding the total number of investors of a lottery project")?;

        // get the lottery
        let lottery = Self::lottery_by_id(&lottery_id);
        // ensure that the project is valid to invest
        ensure!(<system::Module<T>>::block_number() < lottery.expiry, "This lottery is expired.");

        // reserve the amount of money
        <balances::Module<T>>::reserve(&sender, invest_amount)?;

        // change the state of invest related fields
        <InvestAmount<T>>::insert((lottery_id.clone(), sender.clone()), invest_amount.clone());
        <InvestAccounts<T>>::mutate(&lottery_id, |accounts| accounts.push(sender.clone()));

        // add total support count
        <InvestAccountsCount<T>>::insert(lottery_id.clone(), new_investor_count);

        // change the state of invest related fields
        <InvestedLotterysArray<T>>::insert((sender.clone(), invested_lottery_count), lottery_id.clone());
        <InvestedLotterysCount<T>>::insert(&sender, new_invested_lottery_count);
        <InvestedLotterysIndex<T>>::insert((sender.clone(), lottery_id.clone()), invested_lottery_count);

        // get the total amount of the project and add invest_amount
        let amount_of_funding = Self::total_amount_of_invested(&lottery_id);
        let new_amount_of_funding = amount_of_funding + invest_amount;

        // change the total amount of the project has collected
        <LotteryInvestedAmount<T>>::insert(&lottery_id, new_amount_of_funding);

        Ok(())
    }

    pub fn is_lottery_exists(lottery_id: T::Hash) -> bool{
        <Lotterys<T>>::exists(lottery_id)
    }

    pub fn is_lottery_success(lottery_id: T::Hash) -> u64{
        <Lotterys<T>>::get(lottery_id).status
    }

    pub fn get_lottery_owner(lottery_id: T::Hash) -> Option<T::AccountId> {
        <LotteryOwner<T>>::get(lottery_id)
    }

    pub fn get_lottery_total_balance(lottery_id: T::Hash) -> T::Balance{
        <LotteryInvestedAmount<T>>::get(lottery_id)
    }

    pub fn is_investor(lottery_id: T::Hash, from: T::AccountId) -> bool{
        <InvestAmount<T>>::exists((lottery_id, from))
    }

    pub fn get_invested_number(lottery_id: T::Hash) -> u64{
        <InvestAccountsCount<T>>::get(lottery_id)
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok};
	use runtime_primitives::{
		BuildStorage,
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
	}
	impl Trait for Test {
		type Event = ();
	}
	type luckyplus = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(luckyplus::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(luckyplus::something(), Some(42));
		});
	}
}
