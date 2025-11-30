use actix_session::Session;
use actix_web::{get, post, web, HttpResponse, Result};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use uuid::Uuid;

use crate::models::EventStatus;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateEventForm {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct JoinEventForm {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmIdentityForm {
    pub participant_id: String,
}

#[derive(Debug, Serialize)]
pub struct FuzzyMatch {
    pub id: String,
    pub name: String,
    pub score: i64,
}

fn render_template(tera: &Tera, template: &str, context: &Context) -> HttpResponse {
    match tera.render(template, context) {
        Ok(body) => HttpResponse::Ok().content_type("text/html; charset=utf-8").body(body),
        Err(e) => {
            eprintln!("Template error: {}", e);
            HttpResponse::InternalServerError().body("Template rendering error")
        }
    }
}

#[get("/")]
pub async fn index(tera: web::Data<Tera>) -> HttpResponse {
    let context = Context::new();
    render_template(&tera, "index.html", &context)
}

#[get("/create")]
pub async fn create_event_page(tera: web::Data<Tera>) -> HttpResponse {
    let context = Context::new();
    render_template(&tera, "create.html", &context)
}

#[post("/create")]
pub async fn create_event(
    form: web::Form<CreateEventForm>,
    state: web::Data<AppState>,
    tera: web::Data<Tera>,
) -> HttpResponse {
    let name = form.name.trim().to_string();
    if name.is_empty() {
        let mut context = Context::new();
        context.insert("error", "Event name cannot be empty");
        return render_template(&tera, "create.html", &context);
    }

    let event = state.create_event(name);
    
    let mut context = Context::new();
    context.insert("event", &event);
    context.insert("organizer_url", &format!("/event/{}/manage/{}", event.id, event.organizer_token));
    context.insert("invite_url", &format!("/join/{}", event.invite_code));
    render_template(&tera, "event_created.html", &context)
}

#[get("/join/{invite_code}")]
pub async fn join_page(
    path: web::Path<String>,
    state: web::Data<AppState>,
    session: Session,
    tera: web::Data<Tera>,
) -> HttpResponse {
    let invite_code = path.into_inner();
    
    let event = match state.get_event_by_invite_code(&invite_code) {
        Some(e) => e,
        None => {
            let mut context = Context::new();
            context.insert("error", "Invalid invite code");
            return render_template(&tera, "error.html", &context);
        }
    };

    // Check if user already has a cookie for this event
    let session_key = format!("participant_{}", event.id);
    if let Ok(Some(participant_id)) = session.get::<String>(&session_key) {
        if let Ok(pid) = Uuid::parse_str(&participant_id) {
            if event.participants.contains_key(&pid) {
                // Redirect to view assignment
                return HttpResponse::Found()
                    .insert_header(("Location", format!("/event/{}/view", event.id)))
                    .finish();
            }
        }
    }

    let mut context = Context::new();
    context.insert("event", &event);
    context.insert("invite_code", &invite_code);
    
    if event.status == EventStatus::Closed {
        // Event is closed, show identity selection
        context.insert("is_closed", &true);
        render_template(&tera, "join.html", &context)
    } else {
        context.insert("is_closed", &false);
        render_template(&tera, "join.html", &context)
    }
}

#[post("/join/{invite_code}")]
pub async fn join_event(
    path: web::Path<String>,
    form: web::Form<JoinEventForm>,
    state: web::Data<AppState>,
    session: Session,
    tera: web::Data<Tera>,
) -> HttpResponse {
    let invite_code = path.into_inner();
    
    let event = match state.get_event_by_invite_code(&invite_code) {
        Some(e) => e,
        None => {
            let mut context = Context::new();
            context.insert("error", "Invalid invite code");
            return render_template(&tera, "error.html", &context);
        }
    };

    if event.status == EventStatus::Closed {
        let mut context = Context::new();
        context.insert("error", "This event is already closed for new participants");
        return render_template(&tera, "error.html", &context);
    }

    let name = form.name.trim().to_string();
    if name.is_empty() {
        let mut context = Context::new();
        context.insert("event", &event);
        context.insert("invite_code", &invite_code);
        context.insert("is_closed", &false);
        context.insert("error", "Name cannot be empty");
        return render_template(&tera, "join.html", &context);
    }

    let participant_id = match state.add_participant(&event.id, name.clone()) {
        Some(id) => id,
        None => {
            let mut context = Context::new();
            context.insert("error", "Failed to join event");
            return render_template(&tera, "error.html", &context);
        }
    };

    // Store participant ID in session
    let session_key = format!("participant_{}", event.id);
    let _ = session.insert(&session_key, participant_id.to_string());

    let mut context = Context::new();
    context.insert("event", &state.get_event(&event.id).unwrap());
    context.insert("participant_name", &name);
    context.insert("event_url", &format!("/event/{}/view", event.id));
    render_template(&tera, "joined.html", &context)
}

#[get("/event/{event_id}/manage/{organizer_token}")]
pub async fn manage_event(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
    tera: web::Data<Tera>,
) -> HttpResponse {
    let (event_id_str, org_token_str) = path.into_inner();
    
    let event_id = match Uuid::parse_str(&event_id_str) {
        Ok(id) => id,
        Err(_) => {
            let mut context = Context::new();
            context.insert("error", "Invalid event ID");
            return render_template(&tera, "error.html", &context);
        }
    };

    let org_token = match Uuid::parse_str(&org_token_str) {
        Ok(t) => t,
        Err(_) => {
            let mut context = Context::new();
            context.insert("error", "Invalid organizer token");
            return render_template(&tera, "error.html", &context);
        }
    };

    let event = match state.get_event(&event_id) {
        Some(e) => e,
        None => {
            let mut context = Context::new();
            context.insert("error", "Event not found");
            return render_template(&tera, "error.html", &context);
        }
    };

    if event.organizer_token != org_token {
        let mut context = Context::new();
        context.insert("error", "Invalid organizer token");
        return render_template(&tera, "error.html", &context);
    }

    let mut context = Context::new();
    context.insert("event", &event);
    context.insert("organizer_token", &org_token_str);
    context.insert("invite_url", &format!("/join/{}", event.invite_code));
    context.insert("can_close", &(event.participants.len() >= 2));
    render_template(&tera, "manage.html", &context)
}

#[post("/event/{event_id}/close/{organizer_token}")]
pub async fn close_event(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
    tera: web::Data<Tera>,
) -> HttpResponse {
    let (event_id_str, org_token_str) = path.into_inner();
    
    let event_id = match Uuid::parse_str(&event_id_str) {
        Ok(id) => id,
        Err(_) => {
            let mut context = Context::new();
            context.insert("error", "Invalid event ID");
            return render_template(&tera, "error.html", &context);
        }
    };

    let org_token = match Uuid::parse_str(&org_token_str) {
        Ok(t) => t,
        Err(_) => {
            let mut context = Context::new();
            context.insert("error", "Invalid organizer token");
            return render_template(&tera, "error.html", &context);
        }
    };

    match state.close_event(&event_id, &org_token) {
        Ok(_) => {
            HttpResponse::Found()
                .insert_header(("Location", format!("/event/{}/manage/{}", event_id, org_token)))
                .finish()
        }
        Err(e) => {
            let mut context = Context::new();
            context.insert("error", e);
            render_template(&tera, "error.html", &context)
        }
    }
}

#[get("/event/{event_id}/view")]
pub async fn view_assignment(
    path: web::Path<String>,
    session: Session,
    state: web::Data<AppState>,
    tera: web::Data<Tera>,
) -> HttpResponse {
    let event_id_str = path.into_inner();
    
    let event_id = match Uuid::parse_str(&event_id_str) {
        Ok(id) => id,
        Err(_) => {
            let mut context = Context::new();
            context.insert("error", "Invalid event ID");
            return render_template(&tera, "error.html", &context);
        }
    };

    let event = match state.get_event(&event_id) {
        Some(e) => e,
        None => {
            let mut context = Context::new();
            context.insert("error", "Event not found");
            return render_template(&tera, "error.html", &context);
        }
    };

    // Try to get participant from session
    let session_key = format!("participant_{}", event.id);
    if let Ok(Some(participant_id_str)) = session.get::<String>(&session_key) {
        if let Ok(participant_id) = Uuid::parse_str(&participant_id_str) {
            if let Some(participant) = event.participants.get(&participant_id) {
                let mut context = Context::new();
                context.insert("event", &event);
                context.insert("participant", participant);
                
                if event.status == EventStatus::Closed {
                    if let Some(assigned) = event.get_assignment(participant_id) {
                        context.insert("assigned_to", assigned);
                    }
                }
                
                return render_template(&tera, "view_assignment.html", &context);
            }
        }
    }

    // No cookie - redirect to identity page
    HttpResponse::Found()
        .insert_header(("Location", format!("/event/{}/identify", event_id)))
        .finish()
}

#[get("/event/{event_id}/identify")]
pub async fn identify_page(
    path: web::Path<String>,
    state: web::Data<AppState>,
    tera: web::Data<Tera>,
) -> HttpResponse {
    let event_id_str = path.into_inner();
    
    let event_id = match Uuid::parse_str(&event_id_str) {
        Ok(id) => id,
        Err(_) => {
            let mut context = Context::new();
            context.insert("error", "Invalid event ID");
            return render_template(&tera, "error.html", &context);
        }
    };

    let event = match state.get_event(&event_id) {
        Some(e) => e,
        None => {
            let mut context = Context::new();
            context.insert("error", "Event not found");
            return render_template(&tera, "error.html", &context);
        }
    };

    let mut context = Context::new();
    context.insert("event", &event);
    render_template(&tera, "identify.html", &context)
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[get("/event/{event_id}/search")]
pub async fn search_participants(
    path: web::Path<String>,
    query: web::Query<SearchQuery>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let event_id_str = path.into_inner();
    
    let event_id = match Uuid::parse_str(&event_id_str) {
        Ok(id) => id,
        Err(_) => return Ok(HttpResponse::BadRequest().json(Vec::<FuzzyMatch>::new())),
    };

    let event = match state.get_event(&event_id) {
        Some(e) => e,
        None => return Ok(HttpResponse::NotFound().json(Vec::<FuzzyMatch>::new())),
    };

    let matcher = SkimMatcherV2::default();
    let search_term = query.q.to_lowercase();
    
    let mut matches: Vec<FuzzyMatch> = event.participants
        .values()
        .filter_map(|p| {
            matcher.fuzzy_match(&p.name.to_lowercase(), &search_term)
                .map(|score| FuzzyMatch {
                    id: p.id.to_string(),
                    name: p.name.clone(),
                    score,
                })
        })
        .collect();
    
    matches.sort_by(|a, b| b.score.cmp(&a.score));
    matches.truncate(5);

    Ok(HttpResponse::Ok().json(matches))
}

#[post("/event/{event_id}/confirm-identity")]
pub async fn confirm_identity(
    path: web::Path<String>,
    form: web::Form<ConfirmIdentityForm>,
    session: Session,
    state: web::Data<AppState>,
    tera: web::Data<Tera>,
) -> HttpResponse {
    let event_id_str = path.into_inner();
    
    let event_id = match Uuid::parse_str(&event_id_str) {
        Ok(id) => id,
        Err(_) => {
            let mut context = Context::new();
            context.insert("error", "Invalid event ID");
            return render_template(&tera, "error.html", &context);
        }
    };

    let participant_id = match Uuid::parse_str(&form.participant_id) {
        Ok(id) => id,
        Err(_) => {
            let mut context = Context::new();
            context.insert("error", "Invalid participant ID");
            return render_template(&tera, "error.html", &context);
        }
    };

    let event = match state.get_event(&event_id) {
        Some(e) => e,
        None => {
            let mut context = Context::new();
            context.insert("error", "Event not found");
            return render_template(&tera, "error.html", &context);
        }
    };

    if !event.participants.contains_key(&participant_id) {
        let mut context = Context::new();
        context.insert("error", "Participant not found in this event");
        return render_template(&tera, "error.html", &context);
    }

    // Store participant ID in session
    let session_key = format!("participant_{}", event.id);
    let _ = session.insert(&session_key, participant_id.to_string());

    HttpResponse::Found()
        .insert_header(("Location", format!("/event/{}/view", event_id)))
        .finish()
}
