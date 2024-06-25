use crate::contract_types::ContractKind;
use scrypto::prelude::*;

#[blueprint]
mod market_manager {

    struct MarketManager {
        name: String,
        kind: ContractKind,
        minimum: Decimal,
        resource_address: ResourceAddress,
        contracts: KeyValueStore<ComponentAddress, ()>,
        listings: KeyValueStore<String, Option<ComponentAddress>>,
        listings_total: Decimal,
        details: KeyValueStore<String, String>,
    }

    impl MarketManager {
        pub fn new(
            name: String,
            kind: ContractKind,
            minimum: Decimal,
            resource_address: ResourceAddress,
        ) -> Owned<MarketManager> {
            let component = Self {
                name,
                kind,
                minimum,
                resource_address,
                contracts: KeyValueStore::new(),
                listings: KeyValueStore::new(),
                listings_total: dec!(0),
                details: KeyValueStore::new(),
            }
            .instantiate();

            component
        }

        pub fn check_contract(
            &self,
            contract_address: ComponentAddress,
            contract_amount: Decimal,
            contract_resource: ResourceAddress,
        ) {
            let contract = self.contracts.get(&contract_address);
            assert!(contract.is_none(), "[Check Contract]: Already added");
            assert!(contract_amount >= self.minimum, "[Mint]: Less than minimum");
            assert!(
                contract_resource == self.resource_address,
                "[Mint]: Different resource"
            );
        }

        pub fn list(&mut self, contract_address: ComponentAddress) {
            self.contracts.insert(contract_address, ());
            let new_total = self.listings_total + 1;
            self.listings_total = new_total;
            self.listings
                .insert(format!("{new_total}"), Some(contract_address));
        }

        pub fn remove(&mut self, key: String) {
            self.listings.insert(key, None)
        }

        pub fn update(&mut self, name: String, minimum: Decimal, details: HashMap<String, String>) {
            self.name = name;
            self.minimum = minimum;
            for (key, value) in details.iter() {
                self.details.insert(key.to_owned(), value.to_owned());
            }
        }
    }
}
