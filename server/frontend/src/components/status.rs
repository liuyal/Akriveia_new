use common::*;
use crate::util;
use super::root;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::services::interval::{ IntervalTask, IntervalService, };
use yew::prelude::*;
use std::time::Duration;
use std::collections::HashMap;

const POLL_RATE: Duration = Duration::from_millis(1000);

pub enum PageState {
    BeaconStatus,
    UserStatus,
}

pub enum Msg {
    //ChangeRootPage(root::Page),
    ChangeStatus(PageState),

    RequestGetBeacons,
    //RequestGetBeacon(i32),
    RequestGetUsers,
    RequestGetUser(i32),
    RequestGetUsersStatus,
    RequestGetMaps,
    RequestGetMap(i32),

    ResponseGetBeacons(util::Response<Vec<Beacon>>),
    //ResponseGetBeacon(util::Response<Beacon>),
    ResponseGetUsers(util::Response<Vec<TrackedUser>>),
    ResponseGetUsersStatus(util::Response<Vec<RealtimeUserData>>),
    ResponseGetUser(util::Response<TrackedUser>),
    ResponseGetMaps(util::Response<Vec<Map>>),
    ResponseGetMap(util::Response<Map>),
}

pub struct Status {
    state: PageState,
    _change_page: Callback<root::Page>,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    interval_service: IntervalService,
    interval_service_task: Option<IntervalTask>,
    users: HashMap<i32, TrackedUser>,
    beacons: HashMap<i32, Beacon>,
    maps: HashMap<i32, Map>,
    self_link: ComponentLink<Self>,
}

impl Status {
    fn restart_service(&mut self) {
        let callback = match self.state {
            PageState::UserStatus => self.self_link.send_back(|_| Msg::RequestGetUsersStatus),
            PageState::BeaconStatus => self.self_link.send_back(|_| Msg::RequestGetBeacons),
        };
        self.interval_service_task = Some(self.interval_service.spawn(POLL_RATE, callback));
    }
}

#[derive(Properties)]
pub struct StatusProps {
    #[props(required)]
    pub change_page: Callback<root::Page>,
}

impl Component for Status {
    type Message = Msg;
    type Properties = StatusProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetBeacons);
        link.send_self(Msg::RequestGetUsers);
        link.send_self(Msg::RequestGetMaps);
        let mut result = Status {
            state: PageState::UserStatus,
            fetch_service: FetchService::new(),
            interval_service: IntervalService::new(),
            interval_service_task: None,
            users: HashMap::new(),
            beacons: HashMap::new(),
            maps: HashMap::new(),
            fetch_task: None,
            self_link: link,
            _change_page: props.change_page,
        };

        result.restart_service();
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangeStatus(state) => {
                self.state = state;
                self.restart_service();
            }
            /*Msg::ChangeRootPage(page) => {
                self.change_page.emit(page);
            }*/
            Msg::RequestGetMaps => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &maps_url(),
                    self.self_link,
                    Msg::ResponseGetMaps
                );
            },
            Msg::RequestGetMap(id) => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetMap
                );
            },
            Msg::RequestGetBeacons => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &beacons_url(),
                    self.self_link,
                    Msg::ResponseGetBeacons
                );
            },
            /*Msg::RequestGetBeacon(id) => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &beacon_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetBeacon
                );
            },
            Msg::RequestGetBeaconsStatus => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &beacon_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetBeacon
                );
            },*/
            Msg::RequestGetUsers => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &users_url(),
                    self.self_link,
                    Msg::ResponseGetUsers
                );
            },
            Msg::RequestGetUsersStatus => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &users_status_url(),
                    self.self_link,
                    Msg::ResponseGetUsersStatus
                );
            },
            Msg::RequestGetUser(id) => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &user_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetUser
                );
            },
            Msg::ResponseGetMaps(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(maps) => {
                            for map in maps {
                                self.maps.insert(map.id, map);
                            }
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to obtain beacon list");
                }
            },
            Msg::ResponseGetMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(map) => {
                            self.maps.insert(map.id, map);
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to obtain beacon list");
                }
            },
            Msg::ResponseGetBeacons(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(beacons) => {
                            for b in beacons {
                                if let Some(mid) = b.map_id {
                                    if !self.maps.contains_key(&mid) {
                                        let mid = mid.clone();
                                        self.self_link.send_back(move |_: ()| Msg::RequestGetMap(mid));
                                    }
                                }
                                self.beacons.insert(b.id, b);
                            }
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to obtain beacon list");
                }
            },
            /*Msg::ResponseGetBeacon(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(beacon) => {
                            if let Some(mid) = beacon.map_id {
                                if !self.maps.contains_key(&mid) {
                                    let mid = mid.clone();
                                    self.self_link.send_back(move |_: ()| Msg::RequestGetMap(mid));
                                }
                            }
                            self.beacons.insert(beacon.id, beacon);
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to obtain beacon list");
                }
            },*/
            Msg::ResponseGetUser(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(user) => {
                            if let Some(mid) = user.map_id {
                                if !self.maps.contains_key(&mid) {
                                    let mid = mid.clone();
                                    self.self_link.send_back(move |_: ()| Msg::RequestGetMap(mid));
                                }
                            }
                            self.users.insert(user.id, user);
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to obtain beacon list");
                }
            },
            Msg::ResponseGetUsers(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(users) => {
                            for user in users {
                                if let Some(mid) = user.map_id {
                                    if !self.maps.contains_key(&mid) {
                                        let mid = mid.clone();
                                        self.self_link.send_back(move |_: ()| Msg::RequestGetMap(mid));
                                    }
                                }
                                self.users.insert(user.id, user);
                            }
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to obtain beacon list");
                }
            },
            Msg::ResponseGetUsersStatus(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(realtime_users) => {
                            println!("realtime is  {:?}", realtime_users);
                            for ru in realtime_users {
                                match self.users.get_mut(&ru.id) {
                                    Some(u) => {
                                        u.merge(ru);
                                    },
                                    None => {
                                        // just drop the realtime data for now until
                                        // the user object is retrieved, more realtime data
                                        // will come eventually and the UI user likely wont
                                        // notice.
                                        self.self_link
                                            .send_back(move |_: ()| Msg::RequestGetUser(ru.id));
                                    }
                                }
                            }
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to obtain beacon list");
                }
            },
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
}

lazy_static! {
    static ref DEFAULT_MAP: Map = Map::new();
}

impl Status {

    fn beacon_table(&self) -> Html<Self> {
        let mut rows = self.beacons.iter().map(|(_id, beacon)| {
            let map = match beacon.map_id {
                Some(mid) => {
                    match self.maps.get(&mid) {
                        Some(map) => map,
                        None => {
                            &DEFAULT_MAP // render default map until the correct one loads
                        }
                    }
                },
                None => {
                    &DEFAULT_MAP // this beacon doesnt have a map
                }
            };

            html! {
                <tr>
                    <td>{ &beacon.mac_address.to_hex_string() }</td>
                    <td>{ format!("{},{}", &beacon.coordinates.x, &beacon.coordinates.y) }</td>
                    <td>{ &map.name }</td>
                    <td>{ &beacon.name }</td>
                    <td>{ beacon.note.as_ref().unwrap_or(&String::new()) }</td>
                </tr>
            }
        });

        html! {
            <>
                <tr>
                    <td>{ "Mac" }</td>
                    <td>{ "Coordinates" }</td>
                    <td>{ "Floor" }</td>
                    <td>{ "Name" }</td>
                    <td>{ "Note" }</td>
                    <td>{ "Actions" }</td>
                </tr>
                { for rows }
            </>
        }
    }

    fn user_table(&self) -> Html<Self> {
        let mut rows = self.users.iter().map(|(_id, user)| {
            let map = match user.map_id {
                Some(mid) => {
                    match self.maps.get(&mid) {
                        Some(map) => map,
                        None => {
                            &DEFAULT_MAP // render default map until the correct one loads
                        }
                    }
                },
                None => {
                    &DEFAULT_MAP // this beacon doesnt have a map
                }
            };

            html! {
                <tr>
                    <td>{ &user.name }</td>
                    <td>{ format!("{},{}", &user.coordinates.x, &user.coordinates.y) }</td>
                    <td>{ &map.name }</td>
                    <td>{ "test" }</td>
                    <td>{ &user.note.as_ref().unwrap_or(&String::new()) }</td>
                    <td>{ "test" }</td>
                </tr>
            }
        });

        html! {
            <>
                <tr>
                    <td>{ "Name" }</td>
                    <td>{ "Coordinates" }</td>
                    <td>{ "Floor" }</td>
                    <td>{ "Last Seen" }</td>
                    <td>{ "Note" }</td>
                    <td>{ "Actions" }</td>
                </tr>
                { for rows }
            </>
        }
    }
}

impl Renderable<Status> for Status {
    fn view(&self) -> Html<Self> {
        let table = match self.state {
            PageState::BeaconStatus => self.beacon_table(),
            PageState::UserStatus => self.user_table(),
        };

        html! {
            <>
                <p>{ "Status" }</p>
                <button
                    onclick=|_| Msg::ChangeStatus(PageState::UserStatus),
                >
                    {"User Status"}
                </button>
                <button
                    onclick=|_| Msg::ChangeStatus(PageState::BeaconStatus),
                >
                    {"Beacon Status"}
                </button>
                <table>
                    { table }
                </table>
            </>

        }
    }
}
