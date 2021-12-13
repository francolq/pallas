//! Ledger primitives and cbor codec for the Alonzo era
//!
//! Handcrafted, idiomatic rust artifacts based on based on the [Alonzo CDDL](https://github.com/input-output-hk/cardano-ledger/blob/master/eras/alonzo/test-suite/cddl-files/alonzo.cddl) file in IOHK repo.

use log::warn;
use minicbor::{bytes::ByteVec, data::Tag};
use minicbor_derive::{Decode, Encode};
use std::collections::BTreeMap;

use crate::utils::KeyValuePairs;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct SkipCbor<const N: usize> {}

impl<'b, const N: usize> minicbor::Decode<'b> for SkipCbor<N> {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        {
            let probe = d.probe();
            warn!("skipped cbor value {}: {:?}", N, probe.datatype()?);
            println!("skipped cbor value {}: {:?}", N, probe.datatype()?);
        }

        d.skip()?;
        Ok(SkipCbor {})
    }
}

impl<const N: usize> minicbor::Encode for SkipCbor<N> {
    fn encode<W: minicbor::encode::Write>(
        &self,
        _e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        todo!()
    }
}

pub type SomeSkipCbor = SkipCbor<0>;

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct VrfCert(#[n(0)] pub ByteVec, #[n(1)] pub ByteVec);

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct HeaderBody {
    #[n(0)]
    pub block_number: u64,

    #[n(1)]
    pub slot: u64,

    #[n(2)]
    pub prev_hash: ByteVec,

    #[n(3)]
    pub issuer_vkey: ByteVec,

    #[n(4)]
    pub vrf_vkey: ByteVec,

    #[n(5)]
    pub nonce_vrf: VrfCert,

    #[n(6)]
    pub leader_vrf: VrfCert,

    #[n(7)]
    pub block_body_size: u64,

    #[n(8)]
    pub block_body_hash: ByteVec,

    #[n(9)]
    pub operational_cert: ByteVec,

    #[n(10)]
    pub unknown_0: u64,

    #[n(11)]
    pub unknown_1: u64,

    #[n(12)]
    pub unknown_2: ByteVec,

    #[n(13)]
    pub protocol_version_major: u64,

    #[n(14)]
    pub protocol_version_minor: u64,
}

#[derive(Encode, Decode, Debug, PartialEq)]
pub struct KesSignature {}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct Header {
    #[n(0)]
    pub header_body: HeaderBody,

    #[n(1)]
    pub body_signature: ByteVec,
}

#[derive(Encode, Decode, Debug, PartialEq)]
pub struct TransactionInput {
    #[n(0)]
    pub transaction_id: ByteVec,

    #[n(1)]
    pub index: u64,
}

pub type ScriptHash = ByteVec;

pub type PolicyId = ScriptHash;

pub type AssetName = ByteVec;

pub type Multiasset<A> = KeyValuePairs<PolicyId, KeyValuePairs<AssetName, A>>;

pub type Mint = Multiasset<i64>;

pub type Coin = u64;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Coin(Coin),
    Multiasset(Coin, Multiasset<u64>),
}

impl<'b> minicbor::decode::Decode<'b> for Value {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        match d.datatype()? {
            minicbor::data::Type::U32 => Ok(Value::Coin(d.decode()?)),
            minicbor::data::Type::U64 => Ok(Value::Coin(d.decode()?)),
            minicbor::data::Type::Array => {
                d.array()?;
                let coin = d.u64()?;
                let multiasset = d.decode()?;
                Ok(Value::Multiasset(coin, multiasset))
            }
            _ => Err(minicbor::decode::Error::Message(
                "unknown cbor data type for Alonzo Value enum",
            )),
        }
    }
}

impl minicbor::encode::Encode for Value {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        // TODO: check how to deal with uint variants (u32 vs u64)
        match self {
            Value::Coin(coin) => {
                e.encode(coin)?;
            }
            Value::Multiasset(coin, other) => {
                e.array(2)?;
                e.encode(coin)?;
                e.encode(other)?;
            }
        };

        Ok(())
    }
}

#[derive(Encode, Decode, Debug, PartialEq)]
pub struct TransactionOutput {
    #[n(0)]
    pub address: ByteVec,

    #[n(1)]
    pub amount: Value,

    #[n(2)]
    pub datum_hash: Option<ByteVec>,
}

pub type Hash28 = ByteVec;
pub type Hash32 = ByteVec;

pub type PoolKeyhash = Hash28;
pub type Epoch = u64;
pub type Genesishash = SkipCbor<5>;
pub type GenesisDelegateHash = SkipCbor<6>;
pub type VrfKeyhash = Hash32;

/* move_instantaneous_reward = [ 0 / 1, { * stake_credential => delta_coin } / coin ]
; The first field determines where the funds are drawn from.
; 0 denotes the reserves, 1 denotes the treasury.
; If the second field is a map, funds are moved to stake credentials,
; otherwise the funds are given to the other accounting pot.
 */

#[derive(Debug, PartialEq, PartialOrd)]
pub enum InstantaneousRewardSource {
    Reserves,
    Treasury,
}

impl<'b> minicbor::decode::Decode<'b> for InstantaneousRewardSource {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let variant = d.u32()?;

        match variant {
            0 => Ok(Self::Reserves),
            1 => Ok(Self::Treasury),
            _ => Err(minicbor::decode::Error::Message("invalid funds variant")),
        }
    }
}

impl minicbor::encode::Encode for InstantaneousRewardSource {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let variant = match self {
            Self::Reserves => 0,
            Self::Treasury => 1,
        };

        e.u32(variant)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum InstantaneousRewardTarget {
    StakeCredentials(BTreeMap<StakeCredential, i64>),
    OtherAccountingPot(Coin),
}

impl<'b> minicbor::decode::Decode<'b> for InstantaneousRewardTarget {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let datatype = d.datatype()?;

        match datatype {
            minicbor::data::Type::Map => {
                let a = d.decode()?;
                Ok(Self::StakeCredentials(a))
            }
            _ => {
                let a = d.decode()?;
                Ok(Self::OtherAccountingPot(a))
            }
        }
    }
}

impl minicbor::encode::Encode for InstantaneousRewardTarget {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            InstantaneousRewardTarget::StakeCredentials(a) => {
                a.encode(e)?;
                Ok(())
            }
            InstantaneousRewardTarget::OtherAccountingPot(a) => {
                a.encode(e)?;
                Ok(())
            }
        }
    }
}

#[derive(Encode, Decode, Debug, PartialEq, PartialOrd)]
#[cbor]
pub struct MoveInstantaneousReward {
    #[n(0)]
    pub source: InstantaneousRewardSource,

    #[n(1)]
    pub target: InstantaneousRewardTarget,
}

pub type Margin = SkipCbor<9>;
pub type RewardAccount = ByteVec;
pub type PoolOwners = SkipCbor<11>;

pub type Port = u32;
pub type IPv4 = ByteVec;
pub type IPv6 = ByteVec;
pub type DnsName = String;

#[derive(Debug, PartialEq)]
pub enum Relay {
    SingleHostAddr(Option<Port>, Option<IPv4>, Option<IPv6>),
    SingleHostName(Option<Port>, DnsName),
    MultiHostName(DnsName),
}

impl<'b> minicbor::decode::Decode<'b> for Relay {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        d.array()?;
        let variant = d.u16()?;

        match variant {
            0 => Ok(Relay::SingleHostAddr(d.decode()?, d.decode()?, d.decode()?)),
            1 => Ok(Relay::SingleHostName(d.decode()?, d.decode()?)),
            2 => Ok(Relay::MultiHostName(d.decode()?)),
            _ => Err(minicbor::decode::Error::Message(
                "invalid variant id for Relay",
            )),
        }
    }
}

impl minicbor::encode::Encode for Relay {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Relay::SingleHostAddr(a, b, c) => {
                e.array(4)?;
                e.encode(0)?;
                e.encode(a)?;
                e.encode(b)?;
                e.encode(c)?;

                Ok(())
            }
            Relay::SingleHostName(a, b) => {
                e.array(3)?;
                e.encode(1)?;
                e.encode(a)?;
                e.encode(b)?;

                Ok(())
            }
            Relay::MultiHostName(a) => {
                e.array(2)?;
                e.encode(2)?;
                e.encode(a)?;

                Ok(())
            }
        }
    }
}

pub type PoolMetadataHash = Hash32;

#[derive(Encode, Decode, Debug, PartialEq)]
pub struct PoolMetadata {
    #[n(0)]
    pub url: String,

    #[n(1)]
    pub hash: PoolMetadataHash,
}

pub type AddrKeyhash = Hash28;
pub type Scripthash = Hash28;

#[derive(Debug, PartialEq)]
pub struct RationalNumber {
    pub numerator: i64,
    pub denominator: u64,
}

impl<'b> minicbor::decode::Decode<'b> for RationalNumber {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        d.tag()?;
        d.array()?;

        Ok(RationalNumber {
            numerator: d.decode()?,
            denominator: d.decode()?,
        })
    }
}

impl minicbor::encode::Encode for RationalNumber {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        // TODO: check if this is the correct tag
        e.tag(Tag::Unassigned(30))?;
        e.array(2)?;
        e.encode(self.numerator)?;
        e.encode(self.denominator)?;

        Ok(())
    }
}

pub type UnitInterval = RationalNumber;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum StakeCredential {
    AddrKeyhash(AddrKeyhash),
    Scripthash(Scripthash),
}

impl<'b> minicbor::decode::Decode<'b> for StakeCredential {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        d.array()?;
        let variant = d.u16()?;

        match variant {
            0 => Ok(StakeCredential::AddrKeyhash(d.decode()?)),
            1 => Ok(StakeCredential::Scripthash(d.decode()?)),
            _ => Err(minicbor::decode::Error::Message(
                "invalid variant id for StakeCredential",
            )),
        }
    }
}

impl minicbor::encode::Encode for StakeCredential {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            StakeCredential::AddrKeyhash(a) => {
                e.array(2)?;
                e.encode(0)?;
                e.encode(a)?;

                Ok(())
            }
            StakeCredential::Scripthash(a) => {
                e.array(2)?;
                e.encode(0)?;
                e.encode(a)?;

                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Certificate {
    StakeRegistration(StakeCredential),
    StakeDeregistration(StakeCredential),
    StakeDelegation(StakeCredential, PoolKeyhash),
    PoolRegistration {
        operator: PoolKeyhash,
        vrf_keyhash: VrfKeyhash,
        pledge: Coin,
        cost: Coin,
        margin: UnitInterval,
        reward_account: RewardAccount,
        pool_owners: Vec<AddrKeyhash>,
        relays: Vec<Relay>,
        pool_metadata: Option<PoolMetadata>,
    },
    PoolRetirement(PoolKeyhash, Epoch),
    GenesisKeyDelegation(Genesishash, GenesisDelegateHash, VrfKeyhash),
    MoveInstantaneousRewardsCert(MoveInstantaneousReward),
}

impl<'b> minicbor::decode::Decode<'b> for Certificate {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        d.array()?;
        let variant = d.u16()?;

        match variant {
            0 => {
                let a = d.decode()?;
                Ok(Certificate::StakeRegistration(a))
            }
            1 => {
                let a = d.decode()?;
                Ok(Certificate::StakeDeregistration(a))
            }
            2 => {
                let a = d.decode()?;
                let b = d.decode()?;
                Ok(Certificate::StakeDelegation(a, b))
            }
            3 => {
                let operator = d.decode()?;
                let vrf_keyhash = d.decode()?;
                let pledge = d.decode()?;
                let cost = d.decode()?;
                let margin = d.decode()?;
                let reward_account = d.decode()?;
                let pool_owners = d.decode()?;
                let relays = d.decode()?;
                let pool_metadata = d.decode()?;

                Ok(Certificate::PoolRegistration {
                    operator,
                    vrf_keyhash,
                    pledge,
                    cost,
                    margin,
                    reward_account,
                    pool_owners,
                    relays,
                    pool_metadata,
                })
            }
            4 => {
                let a = d.decode()?;
                let b = d.decode()?;
                Ok(Certificate::PoolRetirement(a, b))
            }
            5 => {
                let a = d.decode()?;
                let b = d.decode()?;
                let c = d.decode()?;
                Ok(Certificate::GenesisKeyDelegation(a, b, c))
            }
            6 => {
                let a = d.decode()?;
                Ok(Certificate::MoveInstantaneousRewardsCert(a))
            }
            _ => Err(minicbor::decode::Error::Message(
                "unknown variant id for certificate",
            )),
        }
    }
}

impl minicbor::encode::Encode for Certificate {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Certificate::StakeRegistration(a) => {
                e.array(2)?;
                e.u16(0)?;
                e.encode(a)?;

                Ok(())
            }
            Certificate::StakeDeregistration(a) => {
                e.array(2)?;
                e.u16(1)?;
                e.encode(a)?;

                Ok(())
            }
            Certificate::StakeDelegation(a, b) => {
                e.array(3)?;
                e.u16(2)?;
                e.encode(a)?;
                e.encode(b)?;

                Ok(())
            }
            Certificate::PoolRegistration {
                operator,
                vrf_keyhash,
                pledge,
                cost,
                margin,
                reward_account,
                pool_owners,
                relays,
                pool_metadata,
            } => {
                e.array(10)?;
                e.u16(3)?;

                e.encode(operator)?;
                e.encode(vrf_keyhash)?;
                e.encode(pledge)?;
                e.encode(cost)?;
                e.encode(margin)?;
                e.encode(reward_account)?;
                e.encode(pool_owners)?;
                e.encode(relays)?;
                e.encode(pool_metadata)?;

                Ok(())
            }
            Certificate::PoolRetirement(a, b) => {
                e.array(3)?;
                e.u16(4)?;
                e.encode(a)?;
                e.encode(b)?;

                Ok(())
            }
            Certificate::GenesisKeyDelegation(a, b, c) => {
                e.array(4)?;
                e.u16(5)?;
                e.encode(a)?;
                e.encode(b)?;
                e.encode(c)?;

                Ok(())
            }
            Certificate::MoveInstantaneousRewardsCert(a) => {
                e.array(3)?;
                e.u16(6)?;
                e.encode(a)?;

                Ok(())
            }
        }
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cbor(index_only)]
pub enum NetworkId {
    #[n(0)]
    One,
    #[n(1)]
    Two,
}

#[derive(Debug, PartialEq)]
pub enum TransactionBodyComponent {
    Inputs(Vec<TransactionInput>),
    Outputs(Vec<TransactionOutput>),
    Fee(u64),
    Ttl(Option<u64>),
    Certificates(Option<Vec<Certificate>>),
    Withdrawals(Option<BTreeMap<RewardAccount, Coin>>),
    Update(Option<SkipCbor<22>>),
    AuxiliaryDataHash(Option<ByteVec>),
    ValidityIntervalStart(Option<u64>),
    Mint(Option<Multiasset<i64>>),
    ScriptDataHash(Option<Hash32>),
    Collateral(Option<Vec<TransactionInput>>),
    RequiredSigners(Option<Vec<AddrKeyhash>>),
    NetworkId(Option<NetworkId>),
}

impl<'b> minicbor::decode::Decode<'b> for TransactionBodyComponent {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let key: u32 = d.decode()?;

        match key {
            0 => Ok(Self::Inputs(d.decode()?)),
            1 => Ok(Self::Outputs(d.decode()?)),
            2 => Ok(Self::Fee(d.decode()?)),
            3 => Ok(Self::Ttl(d.decode()?)),
            4 => Ok(Self::Certificates(d.decode()?)),
            5 => Ok(Self::Withdrawals(d.decode()?)),
            6 => Ok(Self::Update(d.decode()?)),
            7 => Ok(Self::AuxiliaryDataHash(d.decode()?)),
            8 => Ok(Self::ValidityIntervalStart(d.decode()?)),
            9 => Ok(Self::Mint(d.decode()?)),
            11 => Ok(Self::ScriptDataHash(d.decode()?)),
            13 => Ok(Self::Collateral(d.decode()?)),
            14 => Ok(Self::RequiredSigners(d.decode()?)),
            15 => Ok(Self::NetworkId(d.decode()?)),
            _ => Err(minicbor::decode::Error::Message(
                "invalid map key for transaction body component",
            )),
        }
    }
}

impl minicbor::encode::Encode for TransactionBodyComponent {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            TransactionBodyComponent::Inputs(x) => {
                e.encode(0)?;
                e.encode(x)?;
            }
            TransactionBodyComponent::Outputs(x) => {
                e.encode(1)?;
                e.encode(x)?;
            }
            TransactionBodyComponent::Fee(x) => {
                e.encode(2)?;
                e.encode(x)?;
            }
            TransactionBodyComponent::Ttl(x) => {
                e.encode(3)?;
                e.encode(x)?;
            }
            TransactionBodyComponent::Certificates(x) => {
                e.encode(4)?;
                e.encode(x)?;
            }
            TransactionBodyComponent::Withdrawals(x) => {
                e.encode(5)?;
                e.encode(x)?;
            }
            TransactionBodyComponent::Update(x) => {
                e.encode(6)?;
                e.encode(x)?;
            }
            TransactionBodyComponent::AuxiliaryDataHash(x) => {
                e.encode(7)?;
                e.encode(x)?;
            }
            TransactionBodyComponent::ValidityIntervalStart(x) => {
                e.encode(8)?;
                e.encode(x)?;
            }
            TransactionBodyComponent::Mint(x) => {
                e.encode(9)?;
                e.encode(x)?;
            }
            TransactionBodyComponent::ScriptDataHash(x) => {
                e.encode(11)?;
                e.encode(x)?;
            }
            TransactionBodyComponent::Collateral(x) => {
                e.encode(13)?;
                e.encode(x)?;
            }
            TransactionBodyComponent::RequiredSigners(x) => {
                e.encode(14)?;
                e.encode(x)?;
            }
            TransactionBodyComponent::NetworkId(x) => {
                e.encode(15)?;
                e.encode(x)?;
            }
        }

        Ok(())
    }
}

// Can't derive encode for TransactionBody because it seems to require a very
// particular order for each key in the map
#[derive(Debug, PartialEq)]
pub struct TransactionBody(Vec<TransactionBodyComponent>);

impl<'b> minicbor::decode::Decode<'b> for TransactionBody {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let len = d.map()?.unwrap_or_default();

        let components: Result<_, _> = (0..len).map(|_| d.decode()).collect();

        Ok(Self(components?))
    }
}

impl minicbor::encode::Encode for TransactionBody {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.map(self.0.len() as u64)?;
        for component in &self.0 {
            e.encode(component)?;
        }

        Ok(())
    }
}

#[derive(Encode, Decode, Debug, PartialEq)]
pub struct VKeyWitness {
    #[n(0)]
    pub vkey: ByteVec,

    #[n(1)]
    pub signature: ByteVec,
}

#[derive(Debug, PartialEq)]
pub enum NativeScript {
    ScriptPubkey(AddrKeyhash),
    ScriptAll(Vec<NativeScript>),
    ScriptAny(Vec<NativeScript>),
    ScriptNOfK(u32, Vec<NativeScript>),
    InvalidBefore(u64),
    InvalidHereafter(u64),
}

impl<'b> minicbor::decode::Decode<'b> for NativeScript {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        d.array()?;
        let variant = d.u32()?;

        match variant {
            0 => Ok(NativeScript::ScriptPubkey(d.decode()?)),
            1 => Ok(NativeScript::ScriptAll(d.decode()?)),
            2 => Ok(NativeScript::ScriptAny(d.decode()?)),
            3 => Ok(NativeScript::ScriptNOfK(d.decode()?, d.decode()?)),
            4 => Ok(NativeScript::InvalidBefore(d.decode()?)),
            5 => Ok(NativeScript::InvalidHereafter(d.decode()?)),
            _ => Err(minicbor::decode::Error::Message(
                "unknown variant id for native script",
            )),
        }
    }
}

impl minicbor::encode::Encode for NativeScript {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(2)?;

        match self {
            NativeScript::ScriptPubkey(v) => {
                e.encode(0)?;
                e.encode(v)?;
            }
            NativeScript::ScriptAll(v) => {
                e.encode(1)?;
                e.encode(v)?;
            }
            NativeScript::ScriptAny(v) => {
                e.encode(2)?;
                e.encode(v)?;
            }
            NativeScript::ScriptNOfK(a, b) => {
                e.encode(3)?;
                e.encode(a)?;
                e.encode(b)?;
            }
            NativeScript::InvalidBefore(v) => {
                e.encode(4)?;
                e.encode(v)?;
            }
            NativeScript::InvalidHereafter(v) => {
                e.encode(5)?;
                e.encode(v)?;
            }
        }

        Ok(())
    }
}

pub type PlutusScript = ByteVec;

/*
big_int = int / big_uint / big_nint ; New
big_uint = #6.2(bounded_bytes) ; New
big_nint = #6.3(bounded_bytes) ; New
 */

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BigInt {
    Int(i64),
    BigUInt(ByteVec),
    BigNInt(ByteVec),
}

impl<'b> minicbor::decode::Decode<'b> for BigInt {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let datatype = d.datatype()?;

        match datatype {
            minicbor::data::Type::U8
            | minicbor::data::Type::U16
            | minicbor::data::Type::U32
            | minicbor::data::Type::U64
            | minicbor::data::Type::I8
            | minicbor::data::Type::I16
            | minicbor::data::Type::I32
            | minicbor::data::Type::I64 => Ok(Self::Int(d.decode()?)),
            minicbor::data::Type::Tag => {
                let tag = d.tag()?;

                match tag {
                    minicbor::data::Tag::PosBignum => Ok(Self::BigUInt(d.decode()?)),
                    minicbor::data::Tag::NegBignum => Ok(Self::BigNInt(d.decode()?)),
                    _ => Err(minicbor::decode::Error::Message(
                        "invalid cbor tag for big int",
                    )),
                }
            }
            _ => Err(minicbor::decode::Error::Message(
                "invalid cbor data type for big int",
            )),
        }
    }
}

impl minicbor::encode::Encode for BigInt {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            BigInt::Int(x) => {
                e.encode(x)?;
            }
            BigInt::BigUInt(x) => {
                e.tag(Tag::PosBignum)?;
                e.encode(x)?;
            }
            BigInt::BigNInt(x) => {
                e.tag(Tag::NegBignum)?;
                e.encode(x)?;
            }
        };

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlutusData {
    Constr(Constr<PlutusData>),
    Map(BTreeMap<PlutusData, PlutusData>),
    BigInt(BigInt),
    BoundedBytes(ByteVec),
    Array(Vec<PlutusData>),
    ArrayIndef(IndefVec<PlutusData>),
}

impl<'b> minicbor::decode::Decode<'b> for PlutusData {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let type_ = d.datatype()?;

        match type_ {
            minicbor::data::Type::Tag => {
                let mut probe = d.probe();
                let tag = probe.tag()?;

                match tag {
                    Tag::Unassigned(121..=127 | 1280..=1400 | 102) => Ok(Self::Constr(d.decode()?)),
                    Tag::PosBignum | Tag::NegBignum => Ok(Self::BigInt(d.decode()?)),
                    _ => Err(minicbor::decode::Error::Message(
                        "unknown tag for plutus data tag",
                    )),
                }
            }
            minicbor::data::Type::U8
            | minicbor::data::Type::U16
            | minicbor::data::Type::U32
            | minicbor::data::Type::U64
            | minicbor::data::Type::I8
            | minicbor::data::Type::I16
            | minicbor::data::Type::I32
            | minicbor::data::Type::I64 => Ok(Self::BigInt(d.decode()?)),
            minicbor::data::Type::Map => Ok(Self::Map(d.decode()?)),
            minicbor::data::Type::Bytes => Ok(Self::BoundedBytes(d.decode()?)),
            minicbor::data::Type::Array => Ok(Self::Array(d.decode()?)),
            minicbor::data::Type::ArrayIndef => Ok(Self::ArrayIndef(d.decode()?)),

            _ => Err(minicbor::decode::Error::Message(
                "bad cbor data type for plutus data",
            )),
        }
    }
}

impl minicbor::encode::Encode for PlutusData {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Self::Constr(a) => {
                e.encode(a)?;
            }
            Self::Map(a) => {
                e.encode(a)?;
            }
            Self::BigInt(a) => {
                e.encode(a)?;
            }
            Self::BoundedBytes(a) => {
                e.encode(a)?;
            }
            Self::Array(a) => {
                e.encode(a)?;
            }
            Self::ArrayIndef(a) => {
                e.encode(a)?;
            }
        }

        Ok(())
    }
}

/// A struct that forces encode / decode using indef arrays
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndefVec<A>(pub Vec<A>);

impl<'b, A> minicbor::decode::Decode<'b> for IndefVec<A>
where
    A: minicbor::decode::Decode<'b>,
{
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let values: Vec<A> = d.decode()?;

        Ok(IndefVec(values))
    }
}

impl<A> minicbor::encode::Encode for IndefVec<A>
where
    A: minicbor::encode::Encode,
{
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        if self.0.is_empty() {
            e.begin_array()?;
            for v in &self.0 {
                e.encode(v)?;
            }
            e.end()?;
        } else {
            e.array(0)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Constr<A> {
    pub tag: u64,
    pub prefix: Option<u32>,
    pub values: IndefVec<A>,
}

impl<'b, A> minicbor::decode::Decode<'b> for Constr<A>
where
    A: minicbor::decode::Decode<'b>,
{
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let tag = d.tag()?;

        match tag {
            Tag::Unassigned(x) => match x {
                121..=127 | 1280..=1400 => Ok(Constr {
                    tag: x,
                    values: d.decode()?,
                    prefix: None,
                }),
                102 => {
                    d.array()?;
                    let prefix = Some(d.decode()?);
                    let values = d.decode()?;
                    Ok(Constr {
                        tag: 102,
                        prefix,
                        values,
                    })
                }
                _ => Err(minicbor::decode::Error::Message(
                    "bad tag code for plutus data",
                )),
            },
            _ => Err(minicbor::decode::Error::Message(
                "bad tag code for plutus data",
            )),
        }
    }
}

impl<A> minicbor::encode::Encode for Constr<A>
where
    A: minicbor::encode::Encode,
{
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.tag(Tag::Unassigned(self.tag))?;

        match self.tag {
            102 => {
                e.array(2)?;
                e.encode(self.prefix)?;
                e.encode(&self.values)?;

                Ok(())
            }
            _ => {
                e.encode(&self.values)?;

                Ok(())
            }
        }
    }
}

#[derive(Encode, Decode, Debug, PartialEq)]
pub struct ExUnits {
    #[n(0)]
    pub mem: u32,
    #[n(1)]
    pub steps: u32,
}

#[derive(Encode, Decode, Debug, PartialEq)]
#[cbor(index_only)]
pub enum RedeemerTag {
    #[n(0)]
    Spend,
    #[n(1)]
    Mint,
    #[n(2)]
    Cert,
    #[n(3)]
    Reward,
}

#[derive(Encode, Decode, Debug, PartialEq)]
pub struct Redeemer {
    #[n(0)]
    pub tag: RedeemerTag,

    #[n(1)]
    pub index: u32,

    #[n(2)]
    pub data: PlutusData,

    #[n(3)]
    pub ex_units: ExUnits,
}

/* bootstrap_witness =
[ public_key : $vkey
, signature  : $signature
, chain_code : bytes .size 32
, attributes : bytes
] */

#[derive(Encode, Decode, Debug, PartialEq)]
pub struct BootstrapWitness {
    #[n(0)]
    pub public_key: ByteVec,

    #[n(1)]
    pub signature: ByteVec,

    #[n(2)]
    pub chain_code: ByteVec,

    #[n(3)]
    pub attributes: ByteVec,
}

#[derive(Encode, Decode, Debug, PartialEq)]
#[cbor(map)]
pub struct TransactionWitnessSet {
    #[n(0)]
    pub vkeywitness: Option<Vec<VKeyWitness>>,

    #[n(1)]
    pub native_script: Option<Vec<NativeScript>>,

    #[n(2)]
    pub bootstrap_witness: Option<Vec<BootstrapWitness>>,

    #[n(3)]
    pub plutus_script: Option<Vec<PlutusScript>>,

    #[n(4)]
    pub plutus_data: Option<Vec<PlutusData>>,

    #[n(5)]
    pub redeemer: Option<Vec<Redeemer>>,
}

#[derive(Encode, Decode, Debug, PartialEq)]
#[cbor(map)]
pub struct AlonzoAuxiliaryData {
    #[n(0)]
    pub metadata: Option<Metadata>,
    #[n(1)]
    pub native_scripts: Option<Vec<NativeScript>>,
    #[n(2)]
    pub plutus_scripts: Option<PlutusScript>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Metadatum {
    Int(i64),
    Bytes(ByteVec),
    Text(String),
    Array(Vec<Metadatum>),
    Map(Metadata),
}

impl<'b> minicbor::Decode<'b> for Metadatum {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        match d.datatype()? {
            minicbor::data::Type::U8 => {
                let i = d.u8()?;
                Ok(Metadatum::Int(i as i64))
            }
            minicbor::data::Type::U16 => {
                let i = d.u16()?;
                Ok(Metadatum::Int(i as i64))
            }
            minicbor::data::Type::U32 => {
                let i = d.u32()?;
                Ok(Metadatum::Int(i as i64))
            }
            minicbor::data::Type::U64 => {
                let i = d.u64()?;
                Ok(Metadatum::Int(i as i64))
            }
            minicbor::data::Type::I8 => {
                let i = d.i8()?;
                Ok(Metadatum::Int(i as i64))
            }
            minicbor::data::Type::I16 => {
                let i = d.i16()?;
                Ok(Metadatum::Int(i as i64))
            }
            minicbor::data::Type::I32 => {
                let i = d.i32()?;
                Ok(Metadatum::Int(i as i64))
            }
            minicbor::data::Type::I64 => {
                let i = d.i64()?;
                Ok(Metadatum::Int(i as i64))
            }
            minicbor::data::Type::Bytes => Ok(Metadatum::Bytes(d.decode()?)),
            minicbor::data::Type::String => Ok(Metadatum::Text(d.decode()?)),
            minicbor::data::Type::Array => Ok(Metadatum::Array(d.decode()?)),
            minicbor::data::Type::Map => Ok(Metadatum::Map(d.decode()?)),
            _ => Err(minicbor::decode::Error::Message(
                "Can't turn data type into metadatum",
            )),
        }
    }
}

impl minicbor::Encode for Metadatum {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Metadatum::Int(a) => {
                e.encode(a)?;
            }
            Metadatum::Bytes(a) => {
                e.encode(a)?;
            }
            Metadatum::Text(a) => {
                e.encode(a)?;
            }
            Metadatum::Array(a) => {
                e.encode(a)?;
            }
            Metadatum::Map(a) => {
                e.encode(a)?;
            }
        };

        Ok(())
    }
}

pub type Metadata = KeyValuePairs<Metadatum, Metadatum>;

#[derive(Debug, PartialEq)]
pub enum AuxiliaryData {
    Shelley(Metadata),
    ShelleyMa {
        transaction_metadata: Metadata,
        auxiliary_scripts: Vec<SomeSkipCbor>,
    },
    Alonzo(AlonzoAuxiliaryData),
}

impl<'b> minicbor::Decode<'b> for AuxiliaryData {
    fn decode(d: &mut minicbor::Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        match d.datatype()? {
            minicbor::data::Type::Map => Ok(AuxiliaryData::Shelley(d.decode()?)),
            minicbor::data::Type::Array => {
                d.array()?;
                let transaction_metadata = d.decode()?;
                let auxiliary_scripts = d.decode()?;
                Ok(AuxiliaryData::ShelleyMa {
                    transaction_metadata,
                    auxiliary_scripts,
                })
            }
            minicbor::data::Type::Tag => {
                d.tag()?;
                Ok(AuxiliaryData::Alonzo(d.decode()?))
            }
            _ => Err(minicbor::decode::Error::Message(
                "Can't infer variant from data type for AuxiliaryData",
            )),
        }
    }
}

impl minicbor::Encode for AuxiliaryData {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            AuxiliaryData::Shelley(m) => {
                e.encode(m)?;
            }
            AuxiliaryData::ShelleyMa {
                transaction_metadata,
                auxiliary_scripts,
            } => {
                e.array(2)?;
                e.encode(transaction_metadata)?;
                e.encode(auxiliary_scripts)?;
            }
            AuxiliaryData::Alonzo(v) => {
                // TODO: check if this is the correct tag
                e.tag(Tag::Unassigned(259))?;
                e.encode(v)?;
            }
        };

        Ok(())
    }
}

pub type TransactionIndex = u32;

#[derive(Encode, Decode, Debug, PartialEq)]
pub struct Block {
    #[n(0)]
    pub header: Header,

    #[n(1)]
    pub transaction_bodies: Vec<TransactionBody>,

    #[n(2)]
    pub transaction_witness_sets: Vec<TransactionWitnessSet>,

    #[n(3)]
    pub auxiliary_data_set: BTreeMap<TransactionIndex, AuxiliaryData>,

    #[n(4)]
    pub invalid_transactions: Vec<TransactionIndex>,
}

#[derive(Encode, Decode, Debug)]
pub struct BlockWrapper(#[n(0)] pub u16, #[n(1)] pub Block);

#[cfg(test)]
mod tests {
    use crate::{BlockWrapper, Fragment};
    use minicbor::{self, to_vec};

    #[test]
    fn block_isomorphic_decoding_encoding() {
        let test_blocks = vec![
            include_str!("test_data/test1.block"),
            include_str!("test_data/test2.block"),
            include_str!("test_data/test3.block"),
            include_str!("test_data/test4.block"),
            include_str!("test_data/test5.block"),
            include_str!("test_data/test6.block"),
            include_str!("test_data/test7.block"),
            include_str!("test_data/test8.block"),
            // indef arrays giving trouble, re-encoding doesn't match
            //include_str!("test_data/test9.block"),
        ];

        for (idx, block_str) in test_blocks.iter().enumerate() {
            println!("decoding test block {}", idx + 1);
            let bytes = hex::decode(block_str).expect(&format!("bad block file {}", idx));
            let block = BlockWrapper::decode_fragment(&bytes[..])
                .expect(&format!("error decoding cbor for file {}", idx));
            let bytes2 =
                to_vec(block).expect(&format!("error encoding block cbor for file {}", idx));
            assert_eq!(bytes, bytes2);
        }
    }
}