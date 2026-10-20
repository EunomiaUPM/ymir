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

use serde::{Deserialize, Serialize};

/// What the Verifier's response URI answers, with `200 OK`, once it has processed a wallet's
/// `direct_post`, be it a presentation or an error (OpenID4VP 1.0 §8.2): a JSON object that may
/// carry where the wallet sends the user next.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct DirectPostResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
}

impl DirectPostResponse {
    pub fn new(redirect_uri: Option<String>) -> Self {
        Self { redirect_uri }
    }
}

#[cfg(test)]
mod tests {
    use super::DirectPostResponse;

    /// Without a redirect the body is an empty object, as the spec allows.
    #[test]
    fn serializes_with_or_without_redirect() {
        let none = serde_json::to_string(&DirectPostResponse::new(None)).unwrap();
        assert_eq!(none, "{}");
        let some = serde_json::to_string(&DirectPostResponse::new(Some("https://x/cb".into())));
        assert_eq!(some.unwrap(), r#"{"redirect_uri":"https://x/cb"}"#);
    }
}
