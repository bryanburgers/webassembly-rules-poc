use serde::Deserialize;

mod reso;

#[no_mangle]
pub extern "C" fn validate() {
    let data: Listing = reso::data();
    #[allow(unused)]
    let previous_data: Option<Listing> = reso::previous_data();
    reso::diagnostic("Starting!");

    if data.list_price <= 0.0 {
        reso::error("ListPrice", "List price must be greater than $0");
    }

    if data.mls_status == MlsStatus::Closed {
        reso::set_required("ClosePrice", true);
        reso::set_display("ClosePrice", true);
    } else {
        reso::set_required("ClosePrice", false);
        reso::set_display("ClosePrice", false);
        reso::set("ClosePrice", serde_json::Value::Null);
    }
}

#[derive(Deserialize)]
struct Listing {
    #[serde(rename = "ListPrice")]
    list_price: f64,
    #[serde(rename = "MlsStatus")]
    mls_status: MlsStatus,
}

#[derive(Deserialize, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
enum MlsStatus {
    Active,
    Pending,
    Closed,
}
