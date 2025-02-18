// Copyright (c) 2023 by Alibaba.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

use crate::attestation::Attest;
use anyhow::*;
use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use kbs_types::{Attestation, Tee};
use log::info;
use mobc::{Manager, Pool};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use self::attestation::{
    attestation_request::RuntimeData, attestation_service_client::AttestationServiceClient,
    AttestationRequest, SetPolicyRequest, Tee as GrpcTee,
};

mod attestation {
    tonic::include_proto!("attestation");
}

pub const DEFAULT_AS_ADDR: &str = "http://127.0.0.1:50004";
pub const DEFAULT_POOL_SIZE: u64 = 100;

pub const COCO_AS_HASH_ALGORITHM: &str = "sha384";

fn to_grpc_tee(tee: Tee) -> GrpcTee {
    match tee {
        Tee::AzSnpVtpm => GrpcTee::AzSnpVtpm,
        Tee::Cca => GrpcTee::Cca,
        Tee::Csv => GrpcTee::Csv,
        Tee::Sample => GrpcTee::Sample,
        Tee::Sev => GrpcTee::Sev,
        Tee::Sgx => GrpcTee::Sgx,
        Tee::Snp => GrpcTee::Snp,
        Tee::Tdx => GrpcTee::Tdx,
        Tee::AzTdxVtpm => GrpcTee::AzTdxVtpm,
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GrpcConfig {
    as_addr: Option<String>,
    pool_size: Option<u64>,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            as_addr: Some(DEFAULT_AS_ADDR.to_string()),
            pool_size: Some(DEFAULT_POOL_SIZE),
        }
    }
}

pub struct GrpcClientPool {
    pool: Mutex<Pool<GrpcManager>>,
}

impl GrpcClientPool {
    pub async fn new(config: GrpcConfig) -> Result<Self> {
        let as_addr = config.as_addr.unwrap_or_else(|| {
            log::info!("Default remote AS address ({DEFAULT_AS_ADDR}) is used");
            DEFAULT_AS_ADDR.to_string()
        });

        let pool_size = config.pool_size.unwrap_or_else(|| {
            log::info!("Default AS connection pool size ({DEFAULT_POOL_SIZE}) is used");
            DEFAULT_POOL_SIZE
        });

        info!("connect to remote AS [{as_addr}] with pool size {pool_size}");
        let manager = GrpcManager { as_addr };
        let pool = Mutex::new(Pool::builder().max_open(pool_size).build(manager));

        Ok(Self { pool })
    }
}

#[async_trait]
impl Attest for GrpcClientPool {
    async fn set_policy(&self, input: &[u8]) -> Result<()> {
        let input = String::from_utf8(input.to_vec()).context("parse SetPolicyInput")?;
        let req = tonic::Request::new(SetPolicyRequest { input });

        let mut client = { self.pool.lock().await.get().await? };

        client
            .set_attestation_policy(req)
            .await
            .map_err(|e| anyhow!("Set Policy Failed: {:?}", e))?;

        Ok(())
    }

    async fn verify(&self, tee: Tee, nonce: &str, attestation: &str) -> Result<String> {
        let attestation: Attestation = serde_json::from_str(attestation)?;

        // TODO: align with the guest-components/kbs-protocol side.
        let runtime_data_plaintext = json!({"tee-pubkey": attestation.tee_pubkey, "nonce": nonce});
        let runtime_data_plaintext = serde_json::to_string(&runtime_data_plaintext)
            .context("CoCo AS client: serialize runtime data failed")?;

        let req = tonic::Request::new(AttestationRequest {
            tee: to_grpc_tee(tee).into(),
            evidence: URL_SAFE_NO_PAD.encode(attestation.tee_evidence),
            runtime_data_hash_algorithm: COCO_AS_HASH_ALGORITHM.into(),
            init_data_hash_algorithm: COCO_AS_HASH_ALGORITHM.into(),
            runtime_data: Some(RuntimeData::StructuredRuntimeData(runtime_data_plaintext)),
            init_data: None,
            policy_ids: vec!["default".to_string()],
        });

        let mut client = { self.pool.lock().await.get().await? };

        let token = client
            .attestation_evaluate(req)
            .await?
            .into_inner()
            .attestation_token;

        Ok(token)
    }
}

pub struct GrpcManager {
    as_addr: String,
}

#[async_trait]
impl Manager for GrpcManager {
    type Connection = AttestationServiceClient<Channel>;
    type Error = tonic::transport::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let connection = AttestationServiceClient::connect(self.as_addr.clone()).await?;
        std::result::Result::Ok(connection)
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        std::result::Result::Ok(conn)
    }
}
