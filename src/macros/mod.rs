/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

/// Declarative macro automatically deriving Serde traits leveraging string transitions.
///
/// Binds target concrete types to [`serde::Serialize`] and [`serde::Deserialize`] pipelines
/// by marshaling data layers via their existing [`std::fmt::Display`] and [`std::str::FromStr`] traits.
///
/// # Examples
/// ```rust
/// impl_serde_via_str!(CustomDid, TargetUrn);
/// ```
#[macro_export]
macro_rules! impl_serde_via_str {
    ( $( $t:ty ),+ $(,)? ) => {
        $(
            impl serde::Serialize for $t {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    // Encodes the instance mapping its canonical string layout
                    serializer.serialize_str(&self.to_string())
                }
            }

            impl<'de> serde::Deserialize<'de> for $t {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    let s: String = serde::Deserialize::deserialize(deserializer)?;

                    // NOTE: To safely reject malformed network strings without panicking,
                    // you can refactor this binding to capture the error into Serde:
                    //
                    // <$t as ::std::str::FromStr>::from_str(&s)
                    //     .map_err(serde::de::Error::custom)

                    let Ok(value) = <$t as ::std::str::FromStr>::from_str(&s);
                    Ok(value)
                }
            }
        )+
    };
}


/// Declarative macro automatically deriving the required SeaORM traits to
/// persist types as strings.
///
/// Binds target concrete types to SeaORM's persistence layer by marshaling
/// database values via their existing [`std::fmt::Display`] and
/// [`std::str::FromStr`] traits.
///
/// # Examples
/// ```ignore
/// impl_serde_via_str!(Kty);
/// impl_seaorm_via_str!(Kty, 32);
/// ```
#[macro_export]
macro_rules! impl_seaorm_via_str {
    ($t:ty, $max_len:expr) => {
        impl From<$t> for sea_orm::sea_query::Value {
            fn from(v: $t) -> Self {
                sea_orm::sea_query::Value::String(Some(Box::new(v.to_string())))
            }
        }

        impl sea_orm::TryGetable for $t {
            fn try_get_by<I: sea_orm::ColIdx>(
                res: &sea_orm::QueryResult,
                idx: I,
            ) -> Result<Self, sea_orm::TryGetError> {
                let s = String::try_get_by(res, idx)?;
                <$t as std::str::FromStr>::from_str(&s).map_err(|e| {
                    sea_orm::TryGetError::DbErr(sea_orm::DbErr::Type(format!("{:?}", e)))
                })
            }
        }

        impl sea_orm::sea_query::ValueType for $t {
            fn try_from(
                v: sea_orm::sea_query::Value,
            ) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
                match v {
                    sea_orm::sea_query::Value::String(Some(s)) => {
                        <$t as std::str::FromStr>::from_str(&s)
                            .map_err(|_| sea_orm::sea_query::ValueTypeErr)
                    }
                    _ => Err(sea_orm::sea_query::ValueTypeErr),
                }
            }
            fn type_name() -> String {
                stringify!($t).to_owned()
            }
            fn array_type() -> sea_orm::sea_query::ArrayType {
                sea_orm::sea_query::ArrayType::String
            }
            fn column_type() -> sea_orm::sea_query::ColumnType {
                sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::N($max_len))
            }
        }

        impl sea_orm::sea_query::Nullable for $t {
            fn null() -> sea_orm::sea_query::Value {
                sea_orm::sea_query::Value::String(None)
            }
        }
    };
}