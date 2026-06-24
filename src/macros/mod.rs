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