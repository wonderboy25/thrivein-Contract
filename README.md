**Reference Url**

https://medium.com/coinmonks/the-freelancers-smart-contract-how-it-works-fda5e1fddf8d

https://github.com/jacksonng77/freelancer/tree/main/contracts

https://github.com/IMEF-FEMI/freelance_payment_protocol

https://www.figma.com/file/5ZOCgzmMprxNzTsbiApC7e/%5BHandoff%5D-ThriveIN?node-id=0-1

**Near Guide**

// Deploy Freelancer contract

near deploy --accountId=thrivein.near --wasmFile out/main.wasm

// Init contract

near call thrivein.near new '{"owner_id": "owner.thrivein.near", "treasury_id": "treasure.thrivein.near"}' --accountId=thrivein.near

// Manager creates a project

near call thrivein.near create_project '{"project_id": "10203222", "freelancer_id": "freelancer1.near"}' --accountId=owner.thrivein.near

// Freelancer adds a schedule on created project (value:3N)

near call thrivein.near add_schedule '{"project_id": "10203222", "short_code": "xxx", "description": "xxx", "value": "3000000000000000000000000"}' --accountId=freelancer1.near

// Client accepts a project

near call thrivein.near accept_project '{"project_id": "10203222"}' --accountId=client1.near

// Client funds on schedule (client fee: 2%)

near call thrivein.near fund_schedule '{"project_id": "10203222", "schedule_id": "1"}' --accountId=client1.near --depositYocto=3062000000000000000000000

// Freelancer starts a schedule

near call thrivein.near start_schedule '{"project_id": "10203222", "schedule_id": "1"}' --accountId=freelancer1.near

// Client approves a schedule

near call thrivein.near approve_schedule '{"project_id": "10203222", "schedule_id": "1"}' --accountId=client1.near

// Freelancer releases funds

near call thrivein.near release_funds_schedule '{"project_id": "10203222", "schedule_id": "1"}' --accountId=freelancer1.near

// Client or freelancer ends a project

near call thrivein.near end_project '{"project_id": "10203222"}' --accountId=client1.near near call thrivein.near end_project '{"project_id": "10203222"}' --accountId=freelancer1.near

-- public view functions

//get a project

near view thrivein.near get_project '{"project_id": "10203222"}'

//get a schedule

near view thrivein.near get_schedule '{"project_id": "10203222", "schedule_id": "1"}'

//get a freelancer address

near view thrivein.near get_freelancer_id '{"project_id": "10203222"}'

//get a client address

near view thrivein.near get_client_id '{"project_id": "10203222"}'

-- onwer functions

//update freelancer address

near call thrivein.near set_freelancer_id '{"project_id": "10203222", "freelancer_id": "freelancer2.near"}' --accountId=owner.thrivein.near

//update client address

near call thrivein.near set_client_id '{"project_id": "10203222", "client_id": "client2.near"}' --accountId=owner.thrivein.near

//update treasury address

near call thrivein.near set_treasury '{"treasury_id": "treasure2.thrivein.near"}' --accountId=owner.thrivein.near
