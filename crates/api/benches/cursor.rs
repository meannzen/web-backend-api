use std::hint::black_box;

use api::cursor;
use api::dtos::user::UserResponse;
use chrono::Utc;
use criterion::{Criterion, criterion_group, criterion_main};
use domain::pagination::SortDirection;
use domain::users::model::SortField;
use uuid::Uuid;

fn sample_user() -> UserResponse {
    let now = Utc::now();
    UserResponse {
        id: Uuid::new_v4(),
        email: "jane.doe@example.com".to_string(),
        first_name: "Jane".to_string(),
        last_name: "Doe".to_string(),
        role: "teacher".to_string(),
        created_at: now,
        updated_at: now,
    }
}

fn bench_cursor(c: &mut Criterion) {
    let user = sample_user();
    let mut group = c.benchmark_group("cursor");

    for (name, field) in [
        ("created_at", SortField::CreatedAt),
        ("email", SortField::Email),
    ] {
        let encoded = cursor::encode(&user, field, SortDirection::Desc);

        group.bench_function(format!("encode/{name}"), |b| {
            b.iter(|| cursor::encode(black_box(&user), black_box(field), SortDirection::Desc))
        });

        group.bench_function(format!("decode/{name}"), |b| {
            b.iter(|| {
                cursor::decode(
                    black_box(encoded.as_str()),
                    black_box(field),
                    SortDirection::Desc,
                )
                .expect("valid cursor")
            })
        });
    }

    group.bench_function("decode/invalid", |b| {
        b.iter(|| {
            cursor::decode(
                black_box("not-a-cursor!!"),
                SortField::Email,
                SortDirection::Asc,
            )
        })
    });

    group.finish();
}

criterion_group!(benches, bench_cursor);
criterion_main!(benches);
