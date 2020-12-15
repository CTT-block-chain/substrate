//! RPC interface for the kp module.

pub use self::gen_client::Client as KpClient;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use kp::AppFinancedData;
use kp_runtime_api::KpApi as KpRuntimeApi;
pub use kp_runtime_api::KpApi as KpRuntimeRpcApi;
use primitives::{AuthAccountId, Balance, BlockNumber, PowerSize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT, SaturatedConversion},
};
use std::sync::Arc;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct StakeToVoteParams<AccountId, Balance> {
    account: AccountId,
    stake: Balance,
}

#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct StakeToVoteResult<Balance> {
    result: Balance,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct QueryCommodityPowerParams {
    app_id: u32,
    cart_id: Bytes,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct QueryDocumentPowerParams {
    app_id: u32,
    doc_id: Bytes,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct QueryLeaderBoardParams {
    app_id: u32,
    model_id: Bytes,
    block: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct LeaderBoardItemRPC<AccountId> {
    cart_id: Bytes,
    power: PowerSize,
    owner: AccountId,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct LeaderBoardResultRPC<AccountId> {
    accounts: Vec<AccountId>,
    board: Vec<LeaderBoardItemRPC<AccountId>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct DocumentPowerRPC {
    doc_type: u8,
    power: PowerSize,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct AppFinanceRecordParams {
    app_id: u32,
    proposal_id: Bytes,
}

#[rpc]
pub trait KpApi<BlockHash, AccountId, Balance, BlockNumber> {
    #[rpc(name = "kp_totalPower")]
    fn total_power(&self, at: Option<BlockHash>) -> Result<PowerSize>;

    #[rpc(name = "kp_accountPower")]
    fn account_power(&self, account: AccountId, at: Option<BlockHash>) -> Result<PowerSize>;

    #[rpc(name = "kp_commodityPower")]
    fn commodity_power(
        &self,
        query: QueryCommodityPowerParams,
        at: Option<BlockHash>,
    ) -> Result<PowerSize>;

    #[rpc(name = "kp_documentPower")]
    fn document_power(
        &self,
        query: QueryDocumentPowerParams,
        at: Option<BlockHash>,
    ) -> Result<DocumentPowerRPC>;

    #[rpc(name = "kp_isCommodityPowerExist")]
    fn is_commodity_power_exist(
        &self,
        query: QueryCommodityPowerParams,
        at: Option<BlockHash>,
    ) -> Result<bool>;

    #[rpc(name = "kp_leaderBoardResult")]
    fn leader_board_result(
        &self,
        query: QueryLeaderBoardParams,
        at: Option<BlockHash>,
    ) -> Result<LeaderBoardResultRPC<AccountId>>;

    #[rpc(name = "kp_stakeToVote")]
    fn stake_to_vote(
        &self,
        params: StakeToVoteParams<AccountId, u64>,
        at: Option<BlockHash>,
    ) -> Result<StakeToVoteResult<u64>>;

    #[rpc(name = "kp_appFinanceRecord")]
    fn app_finance_record(
        &self,
        params: AppFinanceRecordParams,
        at: Option<BlockHash>,
    ) -> Result<AppFinancedData<Balance, BlockNumber>>;
}

/// A struct that implements the `KpApi`.
pub struct Kp<C, M> {
    // If you have more generics, no need to SumStorage<C, M, N, P, ...>
    // just use a tuple like SumStorage<C, (M, N, P, ...)>
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Kp<C, M> {
    /// Create new `Kp` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
        }
    }
}

impl<C, Block> KpApi<<Block as BlockT>::Hash, AuthAccountId, Balance, BlockNumber> for Kp<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: KpRuntimeRpcApi<Block, AuthAccountId, Balance, BlockNumber>,
{
    fn total_power(&self, at: Option<<Block as BlockT>::Hash>) -> Result<PowerSize> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.total_power(&at);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn account_power(
        &self,
        account: AuthAccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<PowerSize> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.account_power(&at, account);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn commodity_power(
        &self,
        query: QueryCommodityPowerParams,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<PowerSize> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let QueryCommodityPowerParams { app_id, cart_id } = query;

        let runtime_api_result = api.commodity_power(&at, app_id, cart_id.to_vec());
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn document_power(
        &self,
        query: QueryDocumentPowerParams,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<DocumentPowerRPC> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let QueryDocumentPowerParams { app_id, doc_id } = query;

        let runtime_api_result = api.document_power(&at, app_id, doc_id.to_vec());
        match runtime_api_result {
            Ok(v) => {
                let converted: DocumentPowerRPC = DocumentPowerRPC {
                    doc_type: v.doc_type.into(),
                    power: v.power,
                };

                Ok(converted)
            }
            Err(e) => {
                Err(RpcError {
                    code: ErrorCode::ServerError(9876), // No real reason for this value
                    message: "Something wrong".into(),
                    data: Some(format!("{:?}", e).into()),
                })
            }
        }
    }

    fn is_commodity_power_exist(
        &self,
        query: QueryCommodityPowerParams,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<bool> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let QueryCommodityPowerParams { app_id, cart_id } = query;

        let runtime_api_result = api.is_commodity_power_exist(&at, app_id, cart_id.to_vec());
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn leader_board_result(
        &self,
        query: QueryLeaderBoardParams,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<LeaderBoardResultRPC<AuthAccountId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let QueryLeaderBoardParams {
            app_id,
            model_id,
            block,
        } = query;

        let runtime_api_result = api.leader_board_result(&at, block, app_id, model_id.to_vec());

        // convert result
        match runtime_api_result {
            Ok(v) => {
                let mut converted: LeaderBoardResultRPC<AuthAccountId> = LeaderBoardResultRPC {
                    accounts: v.accounts,
                    board: vec![],
                };

                for item in v.board {
                    converted.board.push(LeaderBoardItemRPC {
                        cart_id: item.cart_id.into(),
                        power: item.power,
                        owner: item.owner,
                    });
                }
                Ok(converted)
            }
            Err(e) => {
                Err(RpcError {
                    code: ErrorCode::ServerError(9876), // No real reason for this value
                    message: "Something wrong".into(),
                    data: Some(format!("{:?}", e).into()),
                })
            }
        }
    }

    fn stake_to_vote(
        &self,
        params: StakeToVoteParams<AuthAccountId, u64>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<StakeToVoteResult<u64>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

        let StakeToVoteParams { account, stake } = params;
        // here we use u64 because serde has problem to serilize u128, so we lose a defined accuracy
        let balance: Balance = stake.saturated_into();
        let runtime_api_result = api.stake_to_vote(&at, account, balance);

        // convert result
        match runtime_api_result {
            Ok(v) => Ok(StakeToVoteResult {
                result: v.saturated_into(),
            }),
            Err(e) => {
                Err(RpcError {
                    code: ErrorCode::ServerError(9876), // No real reason for this value
                    message: "Something wrong".into(),
                    data: Some(format!("{:?}", e).into()),
                })
            }
        }
    }

    fn app_finance_record(
        &self,
        query: AppFinanceRecordParams,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<AppFinancedData<Balance, BlockNumber>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let AppFinanceRecordParams {
            app_id,
            proposal_id,
        } = query;

        let runtime_api_result = api.app_finance_record(&at, app_id, proposal_id.to_vec());
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
