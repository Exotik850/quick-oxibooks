use quickbooks_types::reports::{
    types::{QBReportParams, QBReportType},
    Report,
};
use ureq::{http::Method, Agent};

use crate::{functions::qb_request, APIResult, QBContext};

pub trait QBReport {
    fn get<T: QBReportType>(
        qb: &QBContext,
        client: &Agent,
        report_type: &T,
        params: Option<T::QueryParams>,
    ) -> APIResult<Self>
    where
        Self: Sized;
}

impl QBReport for Report {
    fn get<T: QBReportType>(
        qb: &QBContext,
        client: &Agent,
        report_type: &T,
        params: Option<T::QueryParams>,
    ) -> APIResult<Self> {
        let path = format!(
            "/v3/company/{}/reports/{}",
            qb.company_id,
            report_type.url_name()
        );
        qb_request(
            qb,
            client,
            Method::GET,
            dbg!(&path),
            None::<&()>,
            Some("application/json"),
            params.as_ref().map(|p| p.params()),
        )
    }
}