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

use serde::Deserialize;

use crate::errors::Errors;

/// What the holder's wallet posts to the verifier's response URI (OID4VP `direct_post`): its
/// presentation or, when it cannot or will not present, an error (Authorization Error Response;
/// e.g. `access_denied` when it lacks the requested credentials).
///
/// The form carries no tag, so the variant is told by its fields (`untagged`); other fields the
/// wallet sends, such as `state`, are ignored.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum VerifyPayload {
    Presentation {
        vp_token: String,
        /// DIF Presentation Exchange submission: OpenID4VP draft 20 only (1.0 has none).
        #[serde(default)]
        presentation_submission: Option<String>,
    },
    Refusal {
        error: String,
        #[serde(default)]
        error_description: Option<String>,
    },
}

impl VerifyPayload {
    /// The presentation's `vp_token`; `None` when the wallet refused.
    pub fn vp_token(&self) -> Option<&str> {
        match self {
            VerifyPayload::Presentation { vp_token, .. } => Some(vp_token),
            VerifyPayload::Refusal { .. } => None,
        }
    }

    /// The wallet's refusal as an error; `None` when it presented.
    pub fn refusal(&self) -> Option<Errors> {
        match self {
            VerifyPayload::Presentation { .. } => None,
            VerifyPayload::Refusal {
                error,
                error_description,
            } => {
                let reason = match error_description {
                    Some(description) => {
                        format!("wallet refused to present ({error}): {description}")
                    }
                    None => format!("wallet refused to present ({error})"),
                };
                Some(Errors::forbidden(reason, None))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VerifyPayload;

    /// Both answers of a wallet arrive as a form; the variant comes from the fields.
    #[test]
    fn reads_a_presentation_or_a_refusal_from_the_form() {
        let presented: VerifyPayload =
            serde_urlencoded::from_str("vp_token=jwt&presentation_submission=%7B%7D&state=s-1")
                .unwrap();
        assert_eq!(presented.vp_token(), Some("jwt"));
        assert!(presented.refusal().is_none());

        let refused: VerifyPayload = serde_urlencoded::from_str(
            "error=access_denied&error_description=no+credential&state=s-1",
        )
        .unwrap();
        assert_eq!(refused.vp_token(), None);
        assert!(refused.refusal().is_some());

        let bare: VerifyPayload = serde_urlencoded::from_str("error=access_denied").unwrap();
        assert!(bare.refusal().is_some());
    }
}
