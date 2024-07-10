use crate::list::list::List;
use crate::types::ContractKind;
use scrypto::prelude::*;

#[blueprint]
mod category {
    struct Category {
        name: String,
        kind: ContractKind,
        minimum: Decimal,
        fee: Decimal,
        resource_address: ResourceAddress,
        list: Owned<List>,
        details: KeyValueStore<String, String>,
    }

    impl Category {
        pub fn new(
            name: String,
            kind: ContractKind,
            minimum: Decimal,
            fee: Decimal,
            resource_address: ResourceAddress,
        ) -> Owned<Category> {
            let component = Self {
                name,
                kind,
                minimum,
                fee,
                resource_address,
                list: List::new(),
                details: KeyValueStore::new(),
            }
            .instantiate();

            component
        }

        pub fn check_contract(
            &self,
            contract_amount: Decimal,
            contract_resource: ResourceAddress,
        ) -> Decimal {
            assert!(contract_amount >= self.minimum, "[Mint]: Less than minimum");
            assert!(
                contract_resource == self.resource_address,
                "[Mint]: Different resource"
            );
            self.fee
        }

        pub fn list(&mut self, address: ComponentAddress) {
            self.list.add(address);
        }

        pub fn remove(&mut self, address: ComponentAddress) {
            self.list.remove(address);
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
