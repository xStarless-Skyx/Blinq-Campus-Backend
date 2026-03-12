use revolt_database::{Snapshot, SnapshotContent, Database, Report, User};
use revolt_database::events::client::EventV1;
use revolt_models::v0::{ReportStatus, ReportedContent, UserReportReason};
use ulid::Ulid;

pub async fn report_blocked_language(
    db: &Database,
    user: &User,
    content: &str,
    matches: &[String],
) {
    let mut context = String::from(
        "Auto report: blocked message for severe language.\n",
    );

    if !matches.is_empty() {
        context.push_str("Matches: ");
        context.push_str(&matches.join(", "));
        context.push('\n');
    }

    let trimmed_content: String = content.chars().take(2000).collect();
    context.push_str("Content: ");
    context.push_str(&trimmed_content);

    let (snapshot_content, files) = match SnapshotContent::generate_from_user(user.clone()) {
        Ok(value) => value,
        Err(err) => {
            log::warn!("Failed to generate user snapshot for auto report: {err}");
            return;
        }
    };

    for file in files {
        if let Err(err) = db.mark_attachment_as_reported(&file).await {
            log::warn!("Failed to mark attachment as reported: {err}");
        }
    }

    let report_id = Ulid::new().to_string();
    let snapshot = Snapshot {
        id: Ulid::new().to_string(),
        report_id: report_id.clone(),
        content: snapshot_content,
    };

    if let Err(err) = db.insert_snapshot(&snapshot).await {
        log::warn!("Failed to insert auto report snapshot: {err}");
        return;
    }

    let report = Report {
        id: report_id,
        author_id: user.id.clone(),
        content: ReportedContent::User {
            id: user.id.clone(),
            report_reason: UserReportReason::SpamAbuse,
            message_id: None,
        },
        additional_context: context,
        status: ReportStatus::Created {},
        notes: String::new(),
    };

    if let Err(err) = db.insert_report(&report).await {
        log::warn!("Failed to insert auto report: {err}");
        return;
    }

    EventV1::ReportCreate(report.into()).global().await;
}
