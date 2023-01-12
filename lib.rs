#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;

#[ink::contract]
mod peer_name_service {
    // use ink_env::hash::Blake2x256;
   
    use ink_storage::collections::HashMap;

    use ink_env::hash::{Blake2x256, HashOutput};
    //use ink_storage:: collections:: Vec;
    use scale::{Decode, Encode};
    // use ink_env::AccountId;
    //  use ink_env::{Hash, AccountId};
    //    use ink_env::Hash;
    // Resolver is the resolved pns value
    // It could be a wallet, contract, IPFS content hash, IPv4, IPv6 etc
    pub type Resolver = AccountId;
    pub type Label = [u8; 32];

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner,
        NotApproved,
        TokenExists,
        TokenNotFound,
        CannotInsert,
        CannotRemove,
        CannotFetchValue,
        NotAllowed,
        UnauthorizedCaller,
        /// Returned if the name already exists upon registration.
        NameAlreadyExists,
        /// Returned if the name not exists upon registration.
        NameNotExists,
        /// Returned if caller is not owner while required to.
        CallerIsNotOwner,
    }

    #[ink(event)]
    pub struct NewOwner {
        node: Hash,
        label: [u8; 32],
        owner: AccountId,
    }

    #[ink(event)]
    pub struct SubNode {
        node: Hash,
        label: [u8; 32],
        subnode:Hash
       
    }

    #[ink(event)]
    pub struct NewResolver {
        node: Hash,
        resolver: Resolver,
    }

    #[ink(event)]
    pub struct NewTTL {
        node: Hash,
        ttl: u64,
    }

    #[ink(event)]
    pub struct Transfer {
        node: Hash,
        owner: AccountId,
    }

    /// Event emitted when admin change manager.
    #[ink(event)]
    pub struct ChangeManager {
        #[ink(topic)]
        _current_manager: Option<AccountId>,
        #[ink(topic)]
        _new_manager: Option<AccountId>,
    }

    /// Emitted whenever a new name is being registered.
    #[ink(event)]
    pub struct Register {
        #[ink(topic)]
        node: Hash,
        #[ink(topic)]
        from: AccountId,
    }

    /// Emitted whenever an address changes.
    #[ink(event)]
    pub struct SetAddress {
        #[ink(topic)]
        name: Hash,
        from: AccountId,
        #[ink(topic)]
        old_address: Option<AccountId>,
        #[ink(topic)]
        new_address: AccountId,
    }

    // /// Emitted whenever a name is being transferred.
    // #[ink(event)]
    // pub struct Transfer {
    //     #[ink(topic)]
    //     name: Hash,
    //     from: AccountId,
    //     #[ink(topic)]
    //     old_owner: Option<AccountId>,
    //     #[ink(topic)]
    //     new_owner: AccountId,
    // }

    #[ink(storage)]
    #[derive(Default)]
    pub struct PeerName {
        records: HashMap<Hash, AccountId>, // mapping of domain name to owner
        resolvers: HashMap<Hash, Resolver>, // mapping of domain name to resolver
        ttls: HashMap<Hash, u64>,          // mapping of domain name to ttl value
        /// stores admin id of contract
        admin: AccountId,

        /// Stores current manager account id of contract
        manager: AccountId,
    }

    impl PeerName {
        #[ink(constructor)]
        pub fn default(_admin: AccountId, _manager: AccountId) -> Self {
            Self {
                records: Default::default(),
                resolvers: Default::default(),
                ttls: Default::default(),
                manager: _manager,
                admin: _admin,
            }
        }

        fn authorized(&self, node: &Hash) -> bool {
            let caller = Self::env().caller();
            let node_owner = self.records.get(&node);
            return match node_owner {
                Some(owner) => {
                    if caller == *owner {
                        true
                    } else {
                        false
                    }
                }
                None => false,
            };
        }

        /// Node exist or note
        #[ink(message)]
        pub fn is_domain_exist(&self, node: Hash) -> bool {
            if self.records.contains_key(&node) {
                true
            } else {
                false
            }
        }

        /// SubNode exist or note
        #[ink(message)]
        pub fn is_subdomain_exist(&self, subnode: Hash) -> bool {
            if self.records.contains_key(&subnode) {
                true
            } else {
                false
            }
        }

        /// Register specific name with caller as owner.
        #[ink(message)]
        pub fn register_domain(
            &mut self,
            node: Hash,
            owner: AccountId,
            resolver: Resolver,
            ttl: u64,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            if caller != self.manager {
                return Err(Error::UnauthorizedCaller);
            };
            if self.records.contains_key(&node) {
                return Err(Error::NameAlreadyExists);
            }
            self.set_record(node, owner, resolver, ttl);
            //   self.records.insert(node, &owner);
            self.env().emit_event(Register { node, from: owner });

            Ok(())
        }

        /// Register specific name with caller as owner.
        #[ink(message)]
        pub fn register_sub_domain(
            &mut self,
            node: Hash,
            label: Label,
            owner: AccountId,
            resolver: Resolver,
            ttl: u64,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            if !self.records.contains_key(&node) {
                return Err(Error::NameNotExists);
            }

            //self.set_subnode_record(node, label, owner, resolver, ttl);

            //   self.records.insert(node, &owner);
            self.env().emit_event(Register { node, from: owner });

            Ok(())
        }

        fn set_record(
            &mut self,
            node: Hash,
            owner: AccountId,
            resolver: Resolver,
            ttl: u64,
        ) -> bool {
            if !self.authorized(&node) {
                return false;
            }

            self.setOwner(node, owner);
            self._setResolverAndTTL(node, resolver, ttl);

            return true;
        }

        // fn set_subnode_record(
        //     &mut self,
        //     node: Hash,
        //     label: Label,
        //     owner: AccountId,
        //     resolver: Resolver,
        //     ttl: u64,
        // ) -> bool {
        //     if !self.authorized(&node) {
        //         return false;
        //     }

        //     let subnode: Hash = self.setSubnodeOwner(node, label, owner);
        //     self._setResolverAndTTL(node, resolver, ttl);

        //     return true;
        // }

        #[ink(message)]
        pub fn setOwner(&mut self, node: Hash, owner: AccountId) -> bool {
            if !self.authorized(&node) {
                return false;
            }

            self.records.insert(node, owner);
            self.env().emit_event(Transfer {
                node: node,
                owner: owner,
            });

            return true;
        }
        /// Current manager of contract
        #[ink(message)]
        pub fn current_manager(&self) -> AccountId {
            self.manager
        }

        /// Admin of the contract
        #[ink(message)]
        pub fn admin(&self) -> AccountId {
            self.admin
        }

        /// Only Admin can change the current manager
        #[ink(message)]
        pub fn change_manager(&mut self, _manager: AccountId) -> Result<(), Error> {
            let caller = self.env().caller();
            if caller != self.admin {
                return Err(Error::UnauthorizedCaller);
            };

            self.manager = _manager;

            self.env().emit_event(ChangeManager {
                _current_manager: Some(self.manager),
                _new_manager: Some(_manager),
            });

            Ok(())
        }

        /// calculate subnode from lable
        #[ink(message)]
        pub fn get_Subnode(&self, node: Hash, label: Label) -> Hash {
            let encodable = (node,label); // Implements `scale::Encode`
            let mut output = <Blake2x256 as HashOutput>::Type::default(); // 256-bit buffer
            ink_env::hash_encoded::<Blake2x256, _>(&encodable, &mut output);
            let subnodehash = Hash::from(output);
            self.env().emit_event(SubNode {
                node: node,
                label: label,
                subnode:subnodehash
        
            });
            subnodehash
            // assert_eq!(output, EXPECTED);
        }

        #[ink(message)]
        pub fn set_Subnodeowner(
            &mut self,
            node: Hash,
            label: Label,
            owner: AccountId,
        ) -> Result<(), Error> {
            // if !self.authorized(&node) {
            //     return Err(Error::UnauthorizedCaller);
            // }

            let subnode = self.get_Subnode(node, label);
            self.records.insert(subnode, owner);
            self.env().emit_event(NewOwner {
                node: node,
                label: label,
                owner: owner,
            });
            Ok(())
        }

        fn _setResolverAndTTL(&mut self, node: Hash, resolver: Resolver, ttl: u64) {
            match self.resolvers.get(&node) {
                Some(node_resolver) => {}
                _ => {
                    self.resolvers.insert(node, resolver);
                    self.env().emit_event(NewResolver {
                        node: node,
                        resolver: resolver,
                    });
                }
            }

            match self.ttls.get(&node) {
                Some(node_ttl) => {}
                _ => {
                    self.ttls.insert(node, ttl);
                    self.env().emit_event(NewTTL {
                        node: node,
                        ttl: ttl,
                    });
                }
            }
        }

        #[ink(message)]
        pub fn owner(&self, node: Hash) -> Option<AccountId> {
            return self.records.get(&node).cloned();
        }

        #[ink(message)]
        pub fn resolver(&self, node: Hash) -> Option<Resolver> {
            return self.resolvers.get(&node).cloned();
        }

        #[ink(message)]
        pub fn ttl(&self, node: Hash) -> u64 {
            return *self.ttls.get(&node).unwrap_or(&0);
        }
    }
}
