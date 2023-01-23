use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, serde_json::json, AccountId, Balance, Promise,
    BorshStorageKey, PanicOnDefault, 
};

const MAX_DIFF: Balance = 1 * 10u128.pow(24);

#[derive(BorshStorageKey, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[derive(PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ScheduleState {
    planned,
    funded,
    started,
    approved,
    released
}

#[derive(BorshStorageKey, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[derive(PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ProjectState {
    initiated,
    accepted,
    closed
}

impl ProjectState {
    pub fn to_text(&self) -> String {
        match self {
            ProjectState::initiated => "initiated".to_string(),
            ProjectState::accepted => "accepted".to_string(),
            ProjectState::closed => "closed".to_string(),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct schedule {
    pub shortCode: String,
    pub description: String,
    pub value: U128,
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
        let this = Self {
            owner_id: owner_id.into(),
            treasury_id: treasury_id.into(),
            freelancerAddress: env::predecessor_account_id(),
            clientAddress: env::predecessor_account_id(), // env::current_account_id()
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
        _value: U128,
    ) {
        self.assert_project_state(ProjectState::initiated);
        self.assert_freelancer();
        self.totalSchedules += 1;
        self.scheduleRegister.insert(
            &(self.totalSchedules),
            &schedule {
                shortCode: _shortCode.clone().into(),
                description: _description.into(),
                scheduleState: ScheduleState::planned,
                value: _value.clone(),
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
        
        assert!(
            self.clientAddress != env::current_account_id(),
            "Error: Client can't be same as contract address"
        );

        self.assert_project_state(ProjectState::initiated);

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
        self.assert_clientorfreelancer();
        self.assert_nomore_funds();
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
        _scheduleID: U64,
    ) {
        self.assert_project_state(ProjectState::accepted);
        self.assert_schedule_state(ScheduleState::planned, _scheduleID.0);
        self.assert_ample_funding(env::attached_deposit() * (10_000 as u128 - self.clientFee as u128)/ 10_000u128, _scheduleID.0);
        self.assert_client();

        let mut schedule_data = self
            .scheduleRegister
            .get(&_scheduleID.0)
            .expect("Error: Schedule does not exist");

        schedule_data.scheduleState = ScheduleState::funded;
        self.scheduleRegister.insert(&_scheduleID.0, &schedule_data);

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
        _scheduleID: U64,
    ) {
        self.assert_project_state(ProjectState::accepted);
        self.assert_schedule_state(ScheduleState::funded, _scheduleID.0);
        self.assert_freelancer();

        let mut schedule_data = self
            .scheduleRegister
            .get(&_scheduleID.0)
            .expect("Error: Schedule does not exist");

        schedule_data.scheduleState = ScheduleState::started;
        self.scheduleRegister.insert(&_scheduleID.0, &schedule_data);

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
        _scheduleID: U64,
    ) {
        self.assert_project_state(ProjectState::accepted);
        self.assert_schedule_state(ScheduleState::started, _scheduleID.0);
        self.assert_client();

        let mut schedule_data = self
            .scheduleRegister
            .get(&_scheduleID.0)
            .expect("Error: Schedule does not exist");

        schedule_data.scheduleState = ScheduleState::approved;
        self.scheduleRegister.insert(&_scheduleID.0, &schedule_data);

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
        _scheduleID: U64,
    ) {
        self.assert_project_state(ProjectState::accepted);
        self.assert_schedule_state(ScheduleState::approved, _scheduleID.0);
        self.assert_freelancer();

        let mut schedule_data = self
            .scheduleRegister
            .get(&_scheduleID.0)
            .expect("Error: Schedule does not exist");
        //send released funds to freelancer
        Promise::new(self.freelancerAddress.clone()).transfer(schedule_data.value.0 * (10_000 as u128 - self.freelancerFee as u128)/ 10_000u128);
        //send extra funds to treasury account
        let extra_funds = env::storage_usage() as u128 * env::storage_byte_cost();
        Promise::new(self.treasury_id.clone()).transfer(env::account_balance() - extra_funds);

        schedule_data.scheduleState = ScheduleState::released;
        self.scheduleRegister.insert(&_scheduleID.0, &schedule_data);

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

    pub fn getScheduleSupply(&self) -> u64 {
        self.scheduleRegister.len()
    }

    pub fn getSchedule(&self, _scheduleID: U64) -> schedule {
        let schedule_data = self
            .scheduleRegister
            .get(&_scheduleID.0)
            .expect("Error: Schedule does not exist");

        schedule_data
    }

    pub fn getFreelancerAddress(&self) -> AccountId {
        self.freelancerAddress.clone()
    }

    pub fn getClientAddress(&self) -> AccountId {
        self.clientAddress.clone()
    }

    pub fn getProjectState(&self) -> String {
        self.projectState.to_text()
    }

    pub fn getBalance() -> Balance {
        env::account_balance() - env::storage_usage() as u128 * env::storage_byte_cost()
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
        assert!(
            self.clientAddress != env::current_account_id(),
            "Error: Client can't be same as contract address"
        );

        assert_eq!(
            env::predecessor_account_id(),
            self.clientAddress,
            "Error: Client only"
        )
    }

    fn assert_clientorfreelancer(&self) {
        assert!(
            self.clientAddress != env::current_account_id(),
            "Error: Client is not defined"
        );

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
            schedule_data.scheduleState == _state,
            "Error: Only selected Schedule"
        );
    }

    fn assert_ample_funding(&self, _funding: u128, _scheduleID: u64) {
        let schedule_data = self
            .scheduleRegister
            .get(&_scheduleID)
            .expect("Error: Schedule does not exist");

        assert!(
            _funding >= schedule_data.value.0,
            "Error: Less money"
        );
    }

    fn assert_nomore_funds(&self) {
        assert!(
            MAX_DIFF >= env::account_balance() - env::storage_usage() as u128 * env::storage_byte_cost(),
            "Error: No more funds"
        );
    }
}
