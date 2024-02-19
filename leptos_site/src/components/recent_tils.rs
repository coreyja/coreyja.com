use std::rc::Rc;

use leptos::*;
use leptos_query::QueryResult;
use leptos_router::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Til {}

#[server(GetRecentTils, "/api")]
async fn get_recent_tils(id: ()) -> Result<Vec<Til>, ServerFnError> {
    use posts::til::TilPost;

    Ok(vec![])
}

fn use_get_recent_tils(
) -> QueryResult<Result<Vec<Til>, ServerFnError>, impl leptos_query::RefetchFn> {
    leptos_query::use_query(|| (), get_recent_tils, Default::default())
}

#[component]
pub fn RecentTils() -> impl IntoView {
    let QueryResult { data, .. } = use_get_recent_tils();

    view! {
      "List of recent TILs"
      <Suspense
        fallback=move || view! {  <p>"Loading..."</p>}
      >
        {move || {
          data.get().map(|data| data
            .iter()
            .map(|src| {
                view! {
                  {format!("{:?}", src)}
                }.into_view()
            })
            .collect_view())
        }
        }
      </Suspense>

    }
}
