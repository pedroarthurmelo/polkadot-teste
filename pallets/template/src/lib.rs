#![cfg_attr(not(feature = "std"), no_std)]

/// Importação do módulo `impls` (caso seja usado para extensões ou lógica adicional).
mod impls;

/// Importações necessárias do framework Substrate.
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{CheckedAdd, AtLeast32BitUnsigned, BlakeTwo256};
use sp_std::prelude::*;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// Estrutura principal do pallet.
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configuração do pallet, definindo os tipos necessários.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Evento genérico para este pallet.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Define o tipo de saldo nativo usado para transações de Kitties.
        type NativeBalance: Inspect<Self::AccountId> + Mutate<Self::AccountId>;

        /// Define o limite máximo de Kitties que um usuário pode possuir.
        #[pallet::constant]
        type MaxKittiesOwned: Get<u32>;
    }

    /// Define o tipo de saldo usado para preços de Kitties.
    pub type BalanceOf<T> =
        <<T as Config>::NativeBalance as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    /// Estrutura representando um Kitty no armazenamento.
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Kitty<T: Config> {
        pub dna: [u8; 32],
        pub owner: T::AccountId,
        pub price: Option<BalanceOf<T>>,
    }

    /// Armazena o contador global de Kitties.
    #[pallet::storage]
    #[pallet::getter(fn kitty_count)]
    pub(super) type CountForKitties<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Mapeia um DNA de Kitty para a sua estrutura.
    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub(super) type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], Kitty<T>>;

    /// Mapeia cada conta para a lista de Kitties que possui.
    #[pallet::storage]
    #[pallet::getter(fn kitties_owned)]
    pub(super) type KittiesOwned<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<[u8; 32], T::MaxKittiesOwned>,
        ValueQuery,
    >;

    /// Eventos do pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Created { owner: T::AccountId },
        Transferred { from: T::AccountId, to: T::AccountId, kitty_id: [u8; 32] },
        PriceSet { owner: T::AccountId, kitty_id: [u8; 32], new_price: Option<BalanceOf<T>> },
        Sold { buyer: T::AccountId, kitty_id: [u8; 32], price: BalanceOf<T> },
    }

    /// Erros do pallet.
    #[pallet::error]
    pub enum Error<T> {
        TooManyKitties,
        DuplicateKitty,
        TooManyOwned,
        TransferToSelf,
        NoKitty,
        NotOwner,
        NotForSale,
        MaxPriceTooLow,
    }

    /// Funções auxiliares do pallet.
    impl<T: Config> Pallet<T> {
        /// Gera um DNA único para o Kitty.
        pub fn gen_dna() -> [u8; 32] {
            let unique_payload = (
                frame_system::Pallet::<T>::parent_hash(),
                frame_system::Pallet::<T>::block_number(),
                frame_system::Pallet::<T>::extrinsic_index(),
                CountForKitties::<T>::get(),
            );
            BlakeTwo256::hash_of(&unique_payload).into()
        }

        /// Realiza a criação do Kitty e adiciona no armazenamento.
        pub fn mint(owner: T::AccountId, dna: [u8; 32]) -> DispatchResult {
            ensure!(!Kitties::<T>::contains_key(dna), Error::<T>::DuplicateKitty);
            let current_count = CountForKitties::<T>::get();
            let new_count = current_count
                .checked_add(1)
                .ok_or(Error::<T>::TooManyKitties)?;

            let mut owned = KittiesOwned::<T>::get(&owner);
            owned
                .try_push(dna)
                .map_err(|_| Error::<T>::TooManyOwned)?;

            Kitties::<T>::insert(
                dna,
                Kitty {
                    dna,
                    owner: owner.clone(),
                    price: None,
                },
            );
            KittiesOwned::<T>::insert(&owner, owned);
            CountForKitties::<T>::put(new_count);

            Self::deposit_event(Event::Created { owner });
            Ok(())
        }
    }

    /// Funções que podem ser chamadas externamente via extrinsics.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Cria um novo Kitty.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let dna = Self::gen_dna();
            Self::mint(who, dna)?;
            Ok(())
        }

        /// Transfere um Kitty para outro usuário.
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn transfer(
            origin: OriginFor<T>,
            to: T::AccountId,
            kitty_id: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // Implemente a lógica de transferência
            Ok(())
        }

        /// Define o preço de um Kitty.
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn set_price(
            origin: OriginFor<T>,
            kitty_id: [u8; 32],
            new_price: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // Implemente a lógica de definição de preço
            Ok(())
        }

        /// Compra um Kitty de outro usuário.
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn buy_kitty(
            origin: OriginFor<T>,
            kitty_id: [u8; 32],
            max_price: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // Implemente a lógica de compra
            Ok(())
        }
    }
}