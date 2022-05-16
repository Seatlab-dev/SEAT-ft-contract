use super::execution_ext::ExecutionExt;
use near_sdk::{AccountId, Balance, Gas};
use near_sdk_sim::{ExecutionResult, UserAccount, ViewResult};
use std::marker::PhantomData;

/// Exposes some functionality from
/// [`near_sdk_sim::ContractAccount`].
pub trait ContractAcc {
    type Contract;
    fn account_id(&self) -> near_sdk::AccountId;
    fn account(&self) -> Option<near_sdk_sim::account::Account>;
    fn user_account(&self) -> &near_sdk_sim::UserAccount;
}

impl<T> ContractAcc for near_sdk_sim::ContractAccount<T> {
    type Contract = T;
    fn account_id(&self) -> near_sdk::AccountId {
        near_sdk_sim::ContractAccount::account_id(self)
    }
    fn account(&self) -> Option<near_sdk_sim::account::Account> {
        near_sdk_sim::ContractAccount::account(self)
    }
    fn user_account(&self) -> &near_sdk_sim::UserAccount {
        &self.user_account
    }
}

pub trait WithAccount<LocalPin> {
    fn with_account(account_id: AccountId) -> Self;
}

pub trait ContractExt: ContractAcc {
    /// Calculates any extra deposit on the contract not required by it's
    /// storage cost.
    #[allow(clippy::let_and_return)]
    fn get_skewed_extra_deposit(&self) -> u128 {
        // TODO: find out why there is such a constant.
        //
        /// Constant observed from empirical tests.
        ///
        /// After the contract transfered all of it's funds (except for
        /// it's storage requirements and the LEEWAY_CONST), it should have
        /// zero funds left, but in practice it appears to still have
        /// this const amount to it.
        ///
        /// I don't know why this happens.
        const SKEWED_CONST: u128 = 669547687500000000;

        let storage_usage: u64 = self.account().unwrap().storage_usage;
        let storage_cost_test: u128 = near_sdk::env::STORAGE_PRICE_PER_BYTE;
        let storage_cost_total: u128 = storage_usage as u128 * storage_cost_test;

        let amount = self.account().unwrap().amount;

        // println!("storage_usage: {}", storage_usage);
        // println!(
        //     "storage_cost_total: {} ={}",
        //     storage_cost_total,
        //     pretty_near(storage_cost_total)
        // );
        // println!(
        //     "extra deposit: {} ={}",
        //     amount - storage_cost_total,
        //     pretty_near(amount - storage_cost_total)
        // );
        let skewed_extra_deposit = amount - storage_cost_total - SKEWED_CONST;
        // println!(
        //     "skewed extra deposit: {}, ={}",
        //     skewed_extra_deposit,
        //     pretty_near(skewed_extra_deposit)
        // );
        skewed_extra_deposit
    }

    /// Calculates any extra deposit on the contract not required by it's
    /// storage cost and transfer to some other account.
    fn transfer_extra_deposit_to(
        &self,
        receiver: &UserAccount,
    ) {
        let skewed_extra = self.get_skewed_extra_deposit();
        // TODO: test when the extra is otherwise still too small
        // (eg. 1 yoctoNear)
        if skewed_extra == 0 {
            // has no extra to transfer
            return;
        }

        // TODO: find out why there is such a constant.
        //
        /// Constant observed from empirical tests.
        ///
        /// Besides the storage requirements, the contracts appear
        /// to require this fixed amount.
        ///
        /// I don't know why this happens.
        const LEEWAY_CONST: u128 = 45306060187500000000;

        // println!("--");
        let storage_usage: u64 = self.account().unwrap().storage_usage;
        let storage_cost_test: u128 = near_sdk::env::STORAGE_PRICE_PER_BYTE;
        let storage_cost_total: u128 = storage_usage as u128 * storage_cost_test;

        let amount = self.account().unwrap().amount;

        if amount < storage_cost_total + LEEWAY_CONST {
            // cancel if the calculation would underflow
            return;
        }

        let extra = amount - storage_cost_total - LEEWAY_CONST;

        // println!("extra deposit: {} ={}", extra, pretty_near(extra));
        self.user_account().transfer(receiver.account_id(), extra);

        let skewed_extra = self.get_skewed_extra_deposit();
        assert_eq!(skewed_extra, 0);
        // println!("--");
    }

    fn debug_json_call<Result>(
        &self,
        caller: &UserAccount,
        method: &str,
        args: near_sdk::serde_json::Value,
        gas: Gas,
        deposit: Balance,
    ) -> Execution<Result>
    where
        Result: near_sdk::serde::de::DeserializeOwned,
    {
        let method_name = method.to_string();
        let contract_id = self.account_id();
        let args_str = near_sdk::serde_json::to_string(&args).unwrap();
        println!("--- call debug ---");
        println!(
            "near call {contract} {method_name} '{args}' --accountId {signer} --gas {gas} --depositYocto {deposit}",
            contract = &contract_id,
            method_name = &method_name,
            args = &args_str,
            signer = caller.account_id(),
            gas = &gas.0,
            deposit = deposit
        );
        let res = self.json_call(caller, method, args, gas, deposit);
        res.pretty_debug();

        use near_sdk_sim::transaction::ExecutionStatus;
        match res.status() {
            ExecutionStatus::Unknown => {
                println!("--- unknown response ---");
            }
            ExecutionStatus::Failure(e) => {
                println!("--- failed response ---");
                println!("{}", e);
            }
            ExecutionStatus::SuccessReceiptId(receipt_id) => {
                println!("--- receipt response ---");
                println!(
                    "{}…",
                    receipt_id.to_string().chars().take(6).collect::<String>()
                );
            }
            ExecutionStatus::SuccessValue(v) => {
                if v.is_empty() {
                    println!("--- js response ---");
                    println!("null");
                } else {
                    match near_sdk::serde_json::from_slice::<near_sdk::serde_json::Value>(&v) {
                        Ok(value) => {
                            println!("--- json response ---");
                            println!(
                                "{}",
                                near_sdk::serde_json::to_string_pretty(&value).unwrap()
                            );
                        }
                        Err(_) => match String::from_utf8(v.clone()) {
                            Ok(s) => {
                                println!("--- utf8 response ---");
                                println!("{}", s);
                            }
                            Err(_) => {
                                println!("--- base64 response ---");
                                let b64 = base64::encode(&v);
                                println!("{}", b64);
                            }
                        },
                    };
                }
            }
        }
        println!("--- end debug ---");
        res
    }

    fn json_call<Result>(
        &self,
        caller: &UserAccount,
        method: &str,
        args: near_sdk::serde_json::Value,
        gas: Gas,
        deposit: Balance,
    ) -> Execution<Result>
    where
        Result: near_sdk::serde::de::DeserializeOwned,
    {
        let tx = Self::pending_tx_json_call(self.account_id(), method, args);
        let res = caller.function_call(tx, gas.0, deposit);
        Execution::new(res)
    }

    fn debug_json_deploy<LocalPin>(
        root: &UserAccount,
        contract_id: &str,
        wasm_bytes: &[u8],
        method: &str,
        args: near_sdk::serde_json::Value,
        gas: Gas,
        deposit: u128,
    ) -> near_sdk_sim::ContractAccount<Self::Contract>
    where
        Self::Contract: WithAccount<LocalPin>,
    {
        let method_name = method.to_string();
        let args_str = near_sdk::serde_json::to_string(&args).unwrap();
        println!("--- deploy debug ---");
        println!(
            "near deploy --wasmFile \"WASM_PATH\" --contractName \"{contract}\" --initFunction \"{method_name}\" --initArgs '{args}' --initGas \"{gas}\", --initDeposit \"{deposit}\"",
            contract = &contract_id,
            method_name = &method_name,
            args = &args_str,
            gas = gas.0,
            deposit = crate::yocto_to_near(deposit)
        );

        Self::json_deploy(root, contract_id, wasm_bytes, method, args, gas, deposit)
    }

    fn json_deploy<LocalPin>(
        root: &UserAccount,
        contract_id: &str,
        wasm_bytes: &[u8],
        method: &str,
        args: near_sdk::serde_json::Value,
        gas: Gas,
        deposit: u128,
        //
    ) -> near_sdk_sim::ContractAccount<Self::Contract>
    where
        Self::Contract: WithAccount<LocalPin>,
    {
        let account_id = near_sdk::AccountId::new_unchecked(contract_id.to_string());
        let __contract = Self::Contract::with_account(account_id.clone());

        let tx = Self::pending_tx_json_call(account_id, method, args);

        near_sdk_sim::ContractAccount {
            user_account: root.deploy_and_initialize(wasm_bytes, tx, deposit, gas.0),
            contract: __contract,
        }
    }

    fn pending_tx_json_call(
        receiver_id: AccountId,
        method: &str,
        args: near_sdk::serde_json::Value,
    ) -> near_sdk::PendingContractTx {
        near_sdk::PendingContractTx::new_from_bytes(
            receiver_id,
            method,
            args.to_string().into_bytes(),
            false,
        )
    }

    fn debug_json_view<Result>(
        &self,
        method: &str,
        args: near_sdk::serde_json::Value,
    ) -> View<Result>
    where
        Result: near_sdk::serde::de::DeserializeOwned,
    {
        let method_name = method.to_string();
        let contract_id = self.account_id();
        let args_str = near_sdk::serde_json::to_string(&args).unwrap();
        println!("--- view debug ---");
        println!(
            "near view {contract} {method_name} '{args}'",
            contract = &contract_id,
            method_name = &method_name,
            args = &args_str,
        );
        let res = self.json_view(method, args);
        res.pretty_debug();

        if res.is_ok() {
            let v = res.unwrap();
            if v.is_empty() {
                println!("--- js response ---");
                println!("null");
            } else {
                match near_sdk::serde_json::from_slice::<near_sdk::serde_json::Value>(&v) {
                    Ok(value) => {
                        println!("--- json response ---");
                        println!(
                            "{}",
                            near_sdk::serde_json::to_string_pretty(&value).unwrap()
                        );
                    }
                    Err(_) => match String::from_utf8(v.clone()) {
                        Ok(s) => {
                            println!("--- utf8 response ---");
                            println!("{}", s);
                        }
                        Err(_) => {
                            println!("--- base64 response ---");
                            let b64 = base64::encode(&v);
                            println!("{}", b64);
                        }
                    },
                };
            }
        } else {
            println!("--- failed response ---");
            let err = res.unwrap_err();
            println!("{} - {:?}", &err, &err);
        }

        println!("--- end debug ---");
        res
    }

    fn json_view<Result>(
        &self,
        method: &str,
        args: near_sdk::serde_json::Value,
    ) -> View<Result>
    where
        Result: near_sdk::serde::de::DeserializeOwned,
    {
        let res = self
            .user_account()
            .view_method_call(Self::pending_tx_json_view(self.account_id(), method, args));
        View::new(res)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn pending_tx_json_view(
        receiver_id: AccountId,
        method: &str,
        args: near_sdk::serde_json::Value,
    ) -> near_sdk::PendingContractTx {
        near_sdk::PendingContractTx::new_from_bytes(
            receiver_id,
            method,
            args.to_string().into_bytes(),
            true,
        )
    }
}

impl<T> ContractExt for T where T: ContractAcc {}

#[must_use]
pub struct View<T> {
    pub inner: ViewResult,
    _result: PhantomData<T>,
}

impl<T> View<T>
where
    T: near_sdk::serde::de::DeserializeOwned,
{
    pub fn new(result: ViewResult) -> Self {
        Self {
            inner: result,
            _result: PhantomData,
        }
    }
    pub fn unwrap_json(&self) -> T {
        self.inner.unwrap_json()
    }
}

impl View<()> {
    pub fn assert_success(&self) {
        assert!(self.inner.is_ok())
    }
}

impl<T> std::ops::Deref for View<T> {
    type Target = ViewResult;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[must_use]
pub struct Execution<T> {
    pub inner: ExecutionResult,
    _result: PhantomData<T>,
}

impl<T> std::ops::Deref for Execution<T> {
    type Target = ExecutionResult;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Execution<T>
where
    T: near_sdk::serde::de::DeserializeOwned,
{
    pub fn new(result: ExecutionResult) -> Self {
        Self {
            inner: result,
            _result: PhantomData,
        }
    }

    pub fn unwrap_json(&self) -> T {
        self.inner.unwrap_json()
    }
}

impl Execution<()> {
    pub fn assert_success(&self) {
        self.inner.assert_success()
    }
}
