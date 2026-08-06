//! Account activities endpoints (`/v2/account/activities`).

use crate::error::Result;
use crate::rest::encode_segment;

use super::{AccountActivitiesRequest, AccountActivity, ActivityType, TradingClient};

impl TradingClient {
    /// `GET /v2/account/activities` — returns account activities (trade and
    /// non-trade) matching the given filters, as a flat list.
    ///
    /// Pagination is manual: the response is a bare JSON array with no
    /// embedded cursor, so follow-up pages are fetched by passing the
    /// previous page's token in [`AccountActivitiesRequest::page_token`]
    /// (with `page_size` at most 100). No automatic page merging is done.
    pub async fn get_account_activities(
        &self,
        req: &AccountActivitiesRequest,
    ) -> Result<Vec<AccountActivity>> {
        self.rest.get("/v2/account/activities", req).await
    }

    /// `GET /v2/account/activities/{activity_type}` — returns account
    /// activities of a single type (e.g. [`ActivityType::Fill`]).
    ///
    /// `activity_types` on the request is redundant here and should be left
    /// unset.
    pub async fn get_account_activities_by_type(
        &self,
        activity_type: ActivityType,
        req: &AccountActivitiesRequest,
    ) -> Result<Vec<AccountActivity>> {
        self.rest
            .get(
                &format!(
                    "/v2/account/activities/{}",
                    encode_segment(activity_type.as_str())
                ),
                req,
            )
            .await
    }
}
