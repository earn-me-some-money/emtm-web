/*
* Emtm-Controller Modules -- Task Control
*/
extern crate chrono;
extern crate emtm_db;
extern crate json;

use actix_web::{web, HttpResponse};
use chrono::Local;

use crate::control::json_objs;
use crate::control::main_control;
use emtm_db::controller::{
    mission_controller::MissionController, user_controller::UserController, Controller,
};
// Model Schemas
use emtm_db::models::missions::{Mission, MissionType, PartState, Participant};
use emtm_db::models::users::{User, UserId};
use emtm_db::search;

const SUPPORT_TASK_KINDS: i8 = 3;

// Task Manage Function Methods

pub fn release_task(data: web::Json<json_objs::ReleaseTaskObj>) -> HttpResponse {
    let mut result_obj = json_objs::MissionOkObj {
        code: true,
        err_message: "".to_string(),
        mid: 0,
    };

    // Init DB Control
    let db_control = Controller::new();

    // Get current user's database-user-id
    let wechat_user_id: UserId = UserId::WechatId(&data.userid);
    let database_user_id = match db_control.get_user_from_identifier(wechat_user_id) {
        Some(User::Cow(cow)) => cow.uid,
        Some(User::Student(stu)) => stu.uid,
        None => -1,
    };

    // Error Checking -- User Existence
    if database_user_id == -1 {
        result_obj.code = false;
        result_obj.err_message = "Error! Cannot find target user in database!".to_string();
        return HttpResponse::Ok().json(result_obj);
    }

    // Define Task_Release error message
    let error_types = [
        "Task Name Duplication",
        "Task Mode Invalid",
        "Task Pay Can not be Negative",
        "Task Time-Limit Invalid",
        "Task Max-Participants Number Should be Positive",
    ];

    let mut error_index = 5;
    let exist_posted_tasks = db_control.get_poster_missions(database_user_id);
    // Check task name duplication
    for task in exist_posted_tasks.iter() {
        if task.name == data.task_name {
            error_index = 0;
        }
    }

    // Check task mode valid
    if data.task_mode >= SUPPORT_TASK_KINDS || data.task_mode < 0 {
        error_index = 1;
    }

    // Check payment positive
    if data.task_pay <= 0 {
        error_index = 2;
    }

    // Check timelimit valid -- cannot before current time
    if !main_control::time_limit_valid(&data.task_time_limit) {
        error_index = 3;
    }

    match data.task_request.max_participants {
        Some(max_parts) if max_parts <= 0 => {
            error_index = 4;
        }
        _ => (),
    };

    if error_index < 5 {
        result_obj.code = false;
        result_obj.err_message = ["Error!", error_types[error_index]].join(" ").to_string();
        return HttpResponse::Ok().json(result_obj);
    } else {
        // Pass all checking, store into db
        let mission = Mission {
            mid: 0,
            poster_uid: database_user_id,
            bounty: data.task_pay,
            risk: data.task_risk,
            name: data.task_name.clone(),
            mission_type: MissionType::from_val(data.task_mode),
            content: data.task_intro.clone(),
            post_time: (Local::now()).naive_local(),
            deadline: main_control::parse_str_to_naive_date_time(&data.task_time_limit),
            participants: vec![],
            max_participants: data.task_request.max_participants,
            min_grade: None,
            max_grade: None,
            school: None,
            min_finished: None,
            major: None,
            min_credit: None,
        };

        if let Err(err) = db_control.add_mission(&mission) {
            result_obj.code = false;
            result_obj.err_message = format!("{}", err);
        }
    }

    if result_obj.code {
        // Set limit timer

        // Get mission's mid

    }

    HttpResponse::Ok().json(result_obj)
}

pub fn release_task_question(_data: web::Json<json_objs::QuestionNaireObj>) -> HttpResponse {
    let result_obj = json_objs::OriginObj {
        code: true,
        err_message: "".to_string(),
    };

    HttpResponse::Ok().json(result_obj)
}

pub fn release_task_transaction(_data: web::Json<json_objs::TransactionObj>) -> HttpResponse {
    let result_obj = json_objs::OriginObj {
        code: true,
        err_message: "".to_string(),
    };

    HttpResponse::Ok().json(result_obj)
}

pub fn release_task_errand(_data: web::Json<json_objs::ErrandObj>) -> HttpResponse {
    let result_obj = json_objs::OriginObj {
        code: true,
        err_message: "".to_string(),
    };

    HttpResponse::Ok().json(result_obj)
}

pub fn check_task(data: web::Json<json_objs::CheckTaskObj>) -> HttpResponse {
    let mut result_obj = json_objs::TaskDetailObj {
        code: false,
        err_message: "".to_string(),
        // Brief description
        mid: None,
        poster_id: None,
        poster_name: None,
        task_state: None,
        task_user_state: None,
        task_name: None,
        task_intro: None,
        task_mode: None,
        task_pay: None,
        task_time_limit: None,
        // More infos
        task_risk: None,
        task_request: None,
        // Accepter and Finisher list
        accept_users: None,
        finish_users: None,
    };

    // Init db-control
    let db_control = Controller::new();

    // Get target user's database-id
    let wechat_user_id: UserId = UserId::WechatId(&data.userid);
    let database_user_id = match db_control.get_user_from_identifier(wechat_user_id) {
        Some(User::Cow(cow)) => cow.uid,
        Some(User::Student(stu)) => stu.uid,
        None => -1,
    };

    // Handle error
    if database_user_id == -1 {
        result_obj.err_message = "Error! Can not find current user in database!".to_string();
        return HttpResponse::Ok().json(result_obj);
    }

    let wechat_poster_id: UserId = UserId::WechatId(&data.poster_id);
    let database_poster_id = match db_control.get_user_from_identifier(wechat_poster_id) {
        Some(User::Cow(cow)) => cow.uid,
        Some(User::Student(stu)) => stu.uid,
        None => -1,
    };

    // Handle error
    if database_poster_id == -1 {
        result_obj.err_message = "Error! Can not find mission poster in database!".to_string();
        return HttpResponse::Ok().json(result_obj);
    } else {
        let wechat_poster_id_1: UserId = UserId::WechatId(&data.poster_id);
        let database_poster_name = match db_control.get_user_from_identifier(wechat_poster_id_1) {
            Some(User::Cow(cow)) => cow.username,
            Some(User::Student(stu)) => stu.username,
            None => "None".to_string(),
        };
        result_obj.poster_name = Some(database_poster_name);
    }

    // Get target mission's mid
    let missions_collection = db_control.get_poster_missions(database_poster_id);

    let mut have_the_mission = false;
    for task in missions_collection.iter() {
        if task.mid == data.task_mid {
            have_the_mission = true;
            // Set mission parameters
            result_obj.task_name = Some(task.name.clone());
            result_obj.task_intro = Some(task.content.clone());
            result_obj.task_mode = Some(task.mission_type.to_val().into());
            result_obj.task_risk = Some(task.risk);
            result_obj.task_pay = Some(task.bounty);
            result_obj.task_time_limit = Some(task.deadline.to_string());
        }
    }

    // Handle error
    if !have_the_mission {
        result_obj.err_message =
            "Error! Target poster haven't release mission with target mid!".to_string();
        return HttpResponse::Ok().json(result_obj);
    }

    // Check target mission time state
    let mut database_mission_error = false;
    let mut over_time = false;
    match db_control.get_mission_from_mid(data.task_mid) {
        Some(mission) => over_time = mission.deadline < (Local::now()).naive_local(),
        None => {
            database_mission_error = true;
        }
    };

    // Handle error
    if database_mission_error {
        result_obj.err_message = "DataBase Error! Can not reach target mission infos!".to_string();
        return HttpResponse::Ok().json(result_obj);
    } else {
        if over_time {
            result_obj.task_state = Some(false);
        } else {
            result_obj.task_state = Some(true);
        }
    }

    // Find participant's finish state
    let participants = db_control.get_mission_participants(data.task_mid);
    for person in participants.iter() {
        // Find person's wechat-id by their database-id
        let database_person_id: UserId = UserId::Uid(person.student_uid);
        let wechat_person_id = match db_control.get_user_from_identifier(database_person_id) {
            Some(User::Student(stu)) => stu.wechat_id,
            Some(User::Cow(_)) => "".to_string(),
            None => "".to_string(),
        };

        // Handle Error
        if wechat_person_id == "".to_string() {
            result_obj.err_message =
                "DataBase Error! Can not reach mission's participants infos!".to_string();
            return HttpResponse::Ok().json(result_obj);
        }

        // Return mission participants states
    }

    // Finish, Set Response Valid
    result_obj.code = true;

    HttpResponse::Ok().json(result_obj)
}

pub fn check_task_self_receive(_data: web::Json<json_objs::UserIdObj>) -> HttpResponse {
    let result_obj = json_objs::GetTasksObj {
        code: true,
        err_message: "".to_string(),
        tasks: vec![],
    };

    HttpResponse::Ok().json(result_obj)
}

pub fn check_task_self_release(_data: web::Json<json_objs::UserIdObj>) -> HttpResponse {
    let result_obj = json_objs::GetTasksObj {
        code: true,
        err_message: "".to_string(),
        tasks: vec![],
    };

    HttpResponse::Ok().json(result_obj)
}

pub fn check_question_naire(_data: web::Json<json_objs::CheckTaskObj>) -> HttpResponse {
    let result_obj = json_objs::AllAnswerObj {
        code: true,
        err_message: "".to_string(),
        answers: vec![],
    };

    HttpResponse::Ok().json(result_obj)
}

pub fn receive_task(data: web::Json<json_objs::ReceiveTaskObj>) -> HttpResponse {
    let mut result_obj = json_objs::OriginObj {
        code: true,
        err_message: "".to_string(),
    };

    // Init DB Control
    let db_control = Controller::new();

    // Find student database-id by wechat-id
    let wechat_user_id: UserId = UserId::WechatId(&data.userid);
    let database_user_id = match db_control.get_user_from_identifier(wechat_user_id) {
        Some(User::Cow(_cow)) => -1,
        Some(User::Student(stu)) => stu.uid,
        None => -1,
    };

    if database_user_id == -1 {
        result_obj.code = false;
        result_obj.err_message = "Error! Cannot find target student in database!".to_string();
        return HttpResponse::Ok().json(result_obj);
    }

    // Check mission participant duplication
    let wechat_releaser_id: UserId = UserId::WechatId(&data.target_userid);
    let target_releaser_id = match db_control.get_user_from_identifier(wechat_releaser_id) {
        Some(User::Cow(cow)) => cow.uid,
        Some(User::Student(stu)) => stu.uid,
        None => -1,
    };

    let missions_collection = db_control.get_poster_missions(target_releaser_id);

    let mut task_mid = -1;
    for task in missions_collection.iter() {
        if task.name == data.target_task {
            task_mid = task.mid;
        }
    }

    if task_mid != -1 {
        // Check duplication
        let participants = db_control.get_mission_participants(task_mid);
        for person in participants.iter() {
            if person.student_uid == database_user_id {
                result_obj.code = false;
                result_obj.err_message = "Error! Task Participant Duplication!".to_string();
                return HttpResponse::Ok().json(result_obj);
            }
        }
        // Check student condition satisify

        // Check task exceed max_participants or not
        let mut find_mission = false;
        match db_control.get_mission_from_mid(task_mid) {
            Some(x) => {
                find_mission = true;
                result_obj.code = match x.max_participants {
                    Some(parts) => parts > (participants.len() as i32),
                    None => true,
                }
            }
            None => {
                result_obj.code = false;
                result_obj.err_message = "Error! Target Mission Not Exist!".to_string();
            }
        };

        if find_mission && !result_obj.code {
            result_obj.err_message = "Error! Target Task Exceed Max Participants Size!".to_string();
        }

        // Pass Checking, store participant into db
        if result_obj.code {
            let new_part_user = vec![Participant {
                student_uid: database_user_id,
                state: PartState::from_val(0),
            }];

            if let Err(err) = db_control.add_participants(task_mid, &new_part_user) {
                result_obj.code = false;
                result_obj.err_message = format!("{}", err);
            }
        }
    } else {
        result_obj.code = false;
        result_obj.err_message =
            "Error! Cannot match the mission name with target releaser!".to_string();
        return HttpResponse::Ok().json(result_obj);
    }

    HttpResponse::Ok().json(result_obj)
}

pub fn search_mission(data: web::Json<json_objs::MissionSearchObj>) -> HttpResponse {
    let mut result_obj = json_objs::SearchResultObj {
        code: true,
        err_message: "".to_string(),
        search_result: vec![],
    };

    let db_control = Controller::new();
    // Search with database-searcher
    let search_result = search::query_mission(&data.keyword);

    let result_vector = match search_result {
        Ok(result) => result,
        Err(_) => vec![],
    };

    // Check search error or not
    if result_vector.len() == 0 {
        result_obj.code = false;
        result_obj.err_message = "Error! Cannot match any mission with search keyword!".to_string();
        return HttpResponse::Ok().json(result_obj);
    }

    // Parse result vector
    for ele in result_vector.iter() {
        // Search mission with element's mid
        match db_control.get_mission_from_mid(ele.0) {
            Some(the_mission) => {
                // Find poster wechat id
                let poster_id: UserId = UserId::Uid(the_mission.poster_uid);
                let poster_wechatid = match db_control.get_user_from_identifier(poster_id) {
                    Some(User::Cow(cow)) => cow.wechat_id,
                    Some(User::Student(stu)) => stu.wechat_id,
                    None => "".to_string(),
                };

                // Check wechat-id successfully get
                if poster_wechatid.len() == 0 {
                    result_obj.code = false;
                    result_obj.err_message =
                        "Error! Cannot get target mission-poster's wechat id!".to_string();
                    return HttpResponse::Ok().json(result_obj);
                }

                // Push new search result into response
                let new_search_result = json_objs::SearchElementObj {
                    mid: ele.0,
                    name: the_mission.name,
                    content: the_mission.content,
                    poster_userid: poster_wechatid,
                    time_limit: the_mission.deadline.to_string(),
                    score: ele.1,
                };
                result_obj.search_result.push(new_search_result);
            }
            None => (),
        };
    }

    HttpResponse::Ok().json(result_obj)
}

pub fn submit_task_cow(_data: web::Json<json_objs::CheckTaskObj>) -> HttpResponse {
    let result_obj = json_objs::OriginObj {
        code: true,
        err_message: "".to_string(),
    };

    HttpResponse::Ok().json(result_obj)
}

pub fn submit_task_stu(_data: web::Json<json_objs::SubmitQuestionNaireObj>) -> HttpResponse {
    let result_obj = json_objs::OriginObj {
        code: true,
        err_message: "".to_string(),
    };

    HttpResponse::Ok().json(result_obj)
}

pub fn get_tasks(_data: web::Json<json_objs::TaskTypeObj>) -> HttpResponse {
    let result_obj = json_objs::GetTasksObj {
        code: true,
        err_message: "".to_string(),
        tasks: vec![],
    };

    HttpResponse::Ok().json(result_obj)
}
