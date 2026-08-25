macro_rules! abilities {
    ($character:ident {
        $($ability:ident {
            cast: ($event:ident, $($param:pat_param| $type:ty),* $(,)?) $body:block $(,
            cooldown: $cooldown:expr)? $(,
            ready: $ready:expr)? $(,)?
        }),* $(,)?
    }) => {
        #[derive(::serde::Serialize, ::serde::Deserialize, PartialEq, Eq, Hash, Clone, Copy, ::strum::EnumString)]
        pub enum AbilityId {$($ability),*}
        $(
            struct $ability;
            impl $ability {
                fn cast($event: ::bevy::ecs::observer::On<$crate::event::Cast::<$ability>>, $($param: $type),*) $body
            }
        )*

        impl $crate::component::alive::player::character::AbilityId for AbilityId {
            fn trigger(&self, entity: ::bevy::ecs::entity::Entity, commands: &mut ::bevy::ecs::system::Commands) {
                match self {$(
                        Self::$ability => commands.trigger($crate::event::Cast::<$ability>::new(entity))
                ),*}
            }

            fn cooldown(&self) -> ::either::Either<::std::time::Duration, bool> {
                use ::either::Either;
                match self {$(
                        Self::$ability => {
                            $(Either::Left($cooldown))?
                            $(Either::Right($ready))?
                        }
                ),*}
            }
        }

        fn add_ability_systems(app: &mut ::bevy::app::App) {
            $(app.add_observer($ability::cast);)*
        }
    }
}

pub mod warrior;
