use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyEvent, MouseEvent};
use futures::StreamExt;
use tokio::sync::mpsc;

pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize,
    Tick,
    Message(AppMessage),
}

use crate::provider::jira::JiraError;
use crate::provider::types::{IssueMetadata, JiraBoard, JiraBoardConfig, JiraChangelogEntry, JiraComment, JiraIssue, JiraProject, JiraTransition, JiraUser};

pub enum AppMessage {
    TokenValidated(Result<JiraUser, JiraError>),
    SearchResults(Result<Vec<JiraProject>, JiraError>),
    BacklogLoaded(u64, Result<Vec<JiraIssue>, JiraError>),
    BoardsForFindLoaded(String, Result<Vec<JiraBoard>, JiraError>),
    BoardDataLoaded(u64, Result<(JiraBoardConfig, Vec<JiraIssue>), JiraError>),
    ColumnOrderLoaded(Result<Vec<String>, JiraError>),
    IssueDetailLoaded(String, Result<(String, IssueMetadata), JiraError>),
    TransitionsLoaded(String, Result<Vec<JiraTransition>, JiraError>),
    CommentsLoaded(String, Result<Vec<JiraComment>, JiraError>),
    TransitionDone(String, Result<(), JiraError>),
    ChangelogLoaded(String, Result<Vec<JiraChangelogEntry>, JiraError>),
    DescriptionUpdated(String, Result<(), JiraError>),
    AssignableUsersLoaded(Result<Vec<JiraUser>, JiraError>),
    EpicsLoaded(Result<Vec<JiraIssue>, JiraError>),
    IssueTypesLoaded(Result<Vec<String>, JiraError>),
    IssueCreated(Result<String, JiraError>),
    ReloadActiveTab,
    IssueFieldUpdated(String, Result<(), JiraError>),
}

pub async fn event_loop(tx: mpsc::UnboundedSender<AppEvent>) -> Result<()> {
    let mut reader = EventStream::new();
    let tick_rate = Duration::from_secs(10);
    let mut tick_interval = tokio::time::interval(tick_rate);

    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                if tx.send(AppEvent::Tick).is_err() {
                    break;
                }
            }
            event = reader.next() => {
                match event {
                    Some(Ok(Event::Key(key))) => {
                        if tx.send(AppEvent::Key(key)).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Event::Mouse(mouse))) => {
                        if tx.send(AppEvent::Mouse(mouse)).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Event::Resize(_, _))) => {
                        if tx.send(AppEvent::Resize).is_err() {
                            break;
                        }
                    }
                    Some(Err(_)) | None => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
