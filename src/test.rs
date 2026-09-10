use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::{Client as TokenClient, StellarAssetClient},
    vec, Address, Env, String,
};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn setup(
    env: &Env,
) -> (
    Address,
    StellarAssetClient<'_>,
    Address,
    NovaEventsContractClient<'_>,
) {
    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_contract.address();
    let token_admin_client = StellarAssetClient::new(env, &token_addr);

    let contract_id = env.register(NovaEventsContract, ());
    let client = NovaEventsContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin, &token_addr);

    (token_addr, token_admin_client, contract_id, client)
}

fn default_tiers(env: &Env) -> Vec<TierInput> {
    vec![
        env,
        TierInput {
            name: String::from_str(env, "General"),
            price: 10_000_000_i128, // 1 USDC
            supply_cap: 100,
        },
        TierInput {
            name: String::from_str(env, "VIP"),
            price: 50_000_000_i128, // 5 USDC
            supply_cap: 20,
        },
    ]
}

fn create_test_event(env: &Env, client: &NovaEventsContractClient, organizer: &Address) -> u32 {
    client.create_event(
        organizer,
        &String::from_str(env, "Stellar Summit"),
        &String::from_str(env, "The biggest Stellar dev conference"),
        &String::from_str(env, "San Francisco"),
        &1_750_000_000_u64,
        &500_000_000_i128,
        &default_tiers(env),
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn test_redeem_ticket() {
    let env = Env::default();
    env.mock_all_auths();

    let (_, token_admin, _, client) = setup(&env);
    let organizer = Address::generate(&env);
    let buyer = Address::generate(&env);

    token_admin.mint(&buyer, &50_000_000_i128);

    let event_id = create_test_event(&env, &client, &organizer);
    let ticket_id = client.buy_ticket(&buyer, &event_id, &0);

    assert!(!client.get_ticket(&event_id, &ticket_id).redeemed);

    client.redeem_ticket(&organizer, &event_id, &ticket_id);

    assert!(client.get_ticket(&event_id, &ticket_id).redeemed);
}

#[test]
fn test_redeem_ticket_nonexistent_event_returns_event_not_found() {
    // Issue #67: redeem_ticket must return EventNotFound for a nonexistent event_id,
    // similar to how test_sponsor_nonexistent_event_fails works (issue #60).
    let env = Env::default();
    env.mock_all_auths();

    let (_, _, _, client) = setup(&env);
    let organizer = Address::generate(&env);

    // No event created — event_id 99 does not exist.
    let result = client.try_redeem_ticket(&organizer, &99, &0);
    assert_eq!(result, Err(Ok(Error::EventNotFound)));
}
