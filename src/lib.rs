use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, serde_json::json, AccountId, Balance,
    BorshStorageKey, CryptoHash, Gas, PanicOnDefault, Promise, Timestamp,
};
use near_sdk::{is_promise_success, promise_result_as_success};
use std::collections::HashMap;

const MAX_DIFF: Balance = 1 * 10u128.pow(24);
#[derive(BorshStorageKey, BorshSerialize)]
pub enum ScheduleState {
    planned,
    funded,
    started,
    approved,
    released
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum ProjectState {
    initiated,
    accepted,
    closed
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct schedule {
    pub shortCode: String,
    pub description: String,
    pub value: u128,
    pub scheduleState : ScheduleState,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub treasury_id: AccountId,
    pub totalSchedules: u64,
    pub freelancerAddress: AccountId,
    pub clientAddress: AccountId,
    pub projectState: ProjectState,
    pub scheduleRegister: UnorderedMap<u64, schedule>,
    pub clientFee: u16,
    pub freelancerFee: u16
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    Schedules
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        treasury_id: AccountId,
    ) -> Self {
        let mut this = Self {
            owner_id: owner_id.into()
            treasury_id: treasury_id.into(),
            freelancerAddress: env::predecessor_account_id(),   
            clientAddress: env::predecessor_account_id(),
            scheduleRegister: UnorderedMap::new(StorageKey::Schedules),
            projectState: ProjectState::initiated,
            totalSchedules: 0,
            clientFee: 200,
            freelancerFee: 300
        };

        this
    }
    
    // Changing treasury & ownership

    #[payable]
    pub fn set_treasury(&mut self, treasury_id: AccountId) {
        self.assert_owner();
        self.treasury_id = treasury_id;
    }

    //Add schedule

    #[payable]
    pub fn addSchedule(
        &mut self,
        _shortCode: String,
        _description: String,
        _value: u128,
    ) {
        assert_project_state(ProjectState::initiated);
        assert_freelancer();

        self.offers.insert(
            &(self.totalSchedules + 1),
            &schedule {
                buyer_id: _shortCode.clone().into(),
                nft_contract_id: _description.into(),
                scheduleState: ScheduleState::planned
                price: _value.clone().into(),
            },
        );

        env::log_str(
            &json!({
                "event": "add_schedule",
                "params": {
                    "shortcode": _shortCode,
                }
            })
            .to_string(),
        );
    }


    #[payable]
    pub fn acceptProject(
        &mut self,
    ) {
        assert_project_state(ProjectState::initiated);

        self.clientAddress = env::predecessor_account_id();
        self.projectState = ProjectState::accepted;
        env::log_str(
            &json!({
                "event": "project_accept",
                "params": {
                    "client_id": self.clientAddress,
                    "freelancer_id": self.freelancerAddress,
                }
            })
            .to_string(),
        );
    }

    #[payable]
    pub fn endProject(
        &mut self,
    ) {
        assert_clientorfreelancer();
        assert_nomore_funds();
        self.projectState = ProjectState::closed;
        env::log_str(
            &json!({
                "event": "project_end",
                "params": {
                    "client_id": self.clientAddress,
                    "freelancer_id": self.freelancerAddress,
                }
            })
            .to_string(),
        );
    }

    #[payable]
    pub fn fundTask(
        &mut self,
        _scheduleID: u64,
    ) {
        assert_project_state(ProjectState::accepted);
        assert_schedule_state(ScheduleState::planned, _scheduleID);
        assert_ample_funding(env::attached_deposit() * (10_000 as u128 - self.clientFee as u128)/ 10_000u128, _scheduleID);
        assert_client();

        let mut schedule_data = self
            .scheduleRegister
            .get(&_scheduleID)
            .expect("Error: Schedule does not exist");

        schedule_data.scheduleState = ScheduleState::funded;
        self.scheduleRegister.insert(&_scheduleID, &schedule_data);

        env::log_str(
            &json!({
                "event": "task_funded",
                "params": {
                    "schedule_id": _scheduleID,
                }
            })
            .to_string(),
        );
    }

    #[payable]
    pub fn startTask(
        &mut self,
        _scheduleID: u64,
    ) {
        assert_project_state(ProjectState::accepted);
        assert_schedule_state(ScheduleState::funded, _scheduleID);
        assert_freelancer();

        let mut schedule_data = self
            .scheduleRegister
            .get(&_scheduleID)
            .expect("Error: Schedule does not exist");

        schedule_data.scheduleState = ScheduleState::started;
        self.scheduleRegister.insert(&_scheduleID, &schedule_data);

        env::log_str(
            &json!({
                "event": "task_started",
                "params": {
                    "schedule_id": _scheduleID,
                }
            })
            .to_string(),
        );
    }

    #[payable]
    pub fn approveTask(
        &mut self,
        _scheduleID: u64,
    ) {
        assert_project_state(ProjectState::accepted);
        assert_schedule_state(ScheduleState::started, _scheduleID);
        assert_client();

        let mut schedule_data = self
            .scheduleRegister
            .get(&_scheduleID)
            .expect("Error: Schedule does not exist");

        schedule_data.scheduleState = ScheduleState::approved;
        self.scheduleRegister.insert(&_scheduleID, &schedule_data);

        env::log_str(
            &json!({
                "event": "task_approved",
                "params": {
                    "schedule_id": _scheduleID,
                }
            })
            .to_string(),
        );
    }

    #[payable]
    pub fn releaseFunds(
        &mut self,
        _scheduleID: u64,
    ) {
        assert_project_state(ProjectState::accepted);
        assert_schedule_state(ScheduleState::approved, _scheduleID);
        assert_freelancer();

        let mut schedule_data = self
            .scheduleRegister
            .get(&_scheduleID)
            .expect("Error: Schedule does not exist");
        //send released funds to freelancer
        Promise::new(self.freelancerAddress.clone()).transfer(schedule_data.value * (10_000 as u128 - self.freelancerFee as u128)/ 10_000u128);
        //send extra funds to treasury account
        Promise::new(self.treasury_id.clone()).transfer(env::account_balance() - env::storage_usage() * env::storage_byte_cost());

        schedule_data.scheduleState = ScheduleState::released;
        self.scheduleRegister.insert(&_scheduleID, &schedule_data);

        env::log_str(
            &json!({
                "event": "task_released",
                "params": {
                    "schedule_id": _scheduleID,
                }
            })
            .to_string(),
        );
    }

    pub fn getBalance() -> Balance {
        env::account_balance() - env::storage_usage() * env::storage_byte_cost()
    }

    // private fn

    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Error: Owner only"
        )
    }

    fn assert_freelancer(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.freelancerAddress,
            "Error: Freelancer only"
        )
    }

    fn assert_client(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.clientAddress,
            "Error: Client only"
        )
    }

    fn assert_clientorfreelancer(&self) {
        assert!(
            self.clientAddress == env::predecessor_account_id() || self.freelancerAddress == env::predecessor_account_id() ,
            "Error: Only Freelancer or Client"
        );
    }

    fn assert_project_state(&self, _state: ProjectState) {
        assert!(
            self.projectState == _state,
            "Error: Only selected Project"
        );
    }

    fn assert_schedule_state(&self, _state: ScheduleState, _scheduleID: u64) {
        assert!(
            _scheduleID <= self.totalSchedules ,
            "Error: Only progress Schedule"
        );
        let schedule_data = self
            .scheduleRegister
            .get(&_scheduleID)
            .expect("Error: Schedule does not exist");

        assert!(
            schedule_data.scheduleState == _state
            "Error: Only selected Schedule"
        );
    }

    fn assert_ample_funding(&self, _funding: u128, _scheduleID: u64) {
        let schedule_data = self
            .scheduleRegister
            .get(&_scheduleID)
            .expect("Error: Schedule does not exist");

        assert!(
            _funding >= schedule_data.value,
            "Error: Less money"
        );
    }

    fn assert_nomore_funds(&self) {
        assert!(
            MAX_DIFF >= env::account_balance() - env::storage_usage() * env::storage_byte_cost(),
            "Error: No more funds"
        );
    }
}
