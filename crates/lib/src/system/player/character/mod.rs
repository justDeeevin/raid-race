/// Abilities for a character.
///
/// Generates:
/// - `enum AbilityId`
/// - [`impl crate::component::alive::player::character::AbilityId for AbilityId`](crate::component::alive::player::character::AbilityId)
/// - Ability structs for [`Cast`](crate::event::Cast) event
/// - `add_ability_systems` function
// TODO: stat boxes
macro_rules! abilities {
    ($($ability:ident {
        cast: ($event:ident, $($param:pat_param| $type:ty),* $(,)?) $body:block,
        description: $description:literal $(,
        cost: $cost:literal)? $(,
        name: $name:literal)? $(,
        cooldown: $cooldown:expr)? $(,
        ready$ready:vis)? $(,)?
    }),* $(,)?) => {
        #[derive(::serde::Serialize, ::serde::Deserialize, PartialEq, Eq, Hash, Clone, Copy, ::strum::EnumString)]
        #[strum(serialize_all = "snake_case")]
        pub enum AbilityId {$(
            #[doc = $description]
            $ability
        ),*}
        $(
            #[doc = $description]
            struct $ability;
            impl $ability {
                fn cast($event: ::bevy::ecs::observer::On<$crate::event::Cast::<$ability>>, $($param: $type),*) $body
            }
        )*

        impl ::std::fmt::Display for AbilityId {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> std::fmt::Result {
                let name = match self {$(
                    Self::$ability => {
                        stringify!($ability) $(;
                        $name)?
                    }),*
                };

                write!(f, "{name}")
            }
        }

        impl $crate::component::alive::player::character::AbilityId for AbilityId {
            fn trigger(&self, entity: ::bevy::ecs::entity::Entity, commands: &mut ::bevy::ecs::system::Commands) {
                match self {$(
                        Self::$ability => commands.trigger($crate::event::Cast::<$ability>::new(entity))
                ),*}
            }

            fn cooldown(&self) -> ::either::Either<::bevy::time::Timer, bool> {
                use ::either::Either;
                match self {$(
                        Self::$ability => {
                            Either::<::bevy::time::Timer, bool>::Right(false) $(;
                                let mut cd = ::bevy::time::Timer::new($cooldown, ::bevy::time::TimerMode::Once);
                                cd.finish();
                                return Either::<::bevy::time::Timer, bool>::Left(cd)
                            )? $(;
                                #[allow(unused)]
                                $ready type T = ();
                                Either::Right(true)
                            )?
                        }
                ),*}
            }

            fn description(&self) -> String {
                match self {$(
                    Self::$ability => $description.into()
                ),*}
            }

            fn cost(&self) -> u16 {
                match self {$(
                    Self::$ability => {
                        0 $(;
                        $cost)?
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
