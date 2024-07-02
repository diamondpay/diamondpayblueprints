use scrypto::prelude::*;

#[blueprint]
mod list {

    struct List {
        all: KeyValueStore<ComponentAddress, String>,
        list: KeyValueStore<String, Option<ComponentAddress>>,
        list_total: Decimal,
    }

    impl List {
        pub fn new() -> Owned<List> {
            Self {
                all: KeyValueStore::new(),
                list: KeyValueStore::new(),
                list_total: dec!(0),
            }
            .instantiate()
        }

        pub fn add(&mut self, address: ComponentAddress) {
            let no_address = self.all.get(&address).is_none();
            assert!(no_address, "[Add]: Already added");

            let new_total = self.list_total + 1;
            self.list_total = new_total;
            let key = format!("{new_total}");
            self.list.insert(key.clone(), Some(address));
            self.all.insert(address, key);
        }

        pub fn remove(&mut self, address: ComponentAddress) {
            let key = self.all.get(&address).unwrap();
            self.list.insert(key.clone(), None);
        }
    }
}
