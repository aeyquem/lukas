use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
#[error("owner workspace bootstrap failed")]
pub struct BootstrapError;

#[derive(Clone, Copy)]
struct StarterCategory {
    slug: &'static str,
    name: &'static str,
    sort_order: i16,
}

const STARTER_CATEGORIES: [StarterCategory; 15] = [
    StarterCategory {
        slug: "vivienda",
        name: "Vivienda",
        sort_order: 1,
    },
    StarterCategory {
        slug: "alimentacion",
        name: "Alimentación",
        sort_order: 2,
    },
    StarterCategory {
        slug: "transporte",
        name: "Transporte",
        sort_order: 3,
    },
    StarterCategory {
        slug: "servicios-comunicaciones",
        name: "Servicios y comunicaciones",
        sort_order: 4,
    },
    StarterCategory {
        slug: "salud",
        name: "Salud",
        sort_order: 5,
    },
    StarterCategory {
        slug: "educacion",
        name: "Educación",
        sort_order: 6,
    },
    StarterCategory {
        slug: "hogar",
        name: "Hogar",
        sort_order: 7,
    },
    StarterCategory {
        slug: "vestuario-cuidado-personal",
        name: "Vestuario y cuidado personal",
        sort_order: 8,
    },
    StarterCategory {
        slug: "restaurantes-ocio",
        name: "Restaurantes y ocio",
        sort_order: 9,
    },
    StarterCategory {
        slug: "suscripciones",
        name: "Suscripciones",
        sort_order: 10,
    },
    StarterCategory {
        slug: "seguros-impuestos",
        name: "Seguros e impuestos",
        sort_order: 11,
    },
    StarterCategory {
        slug: "mascotas",
        name: "Mascotas",
        sort_order: 12,
    },
    StarterCategory {
        slug: "regalos-donaciones",
        name: "Regalos y donaciones",
        sort_order: 13,
    },
    StarterCategory {
        slug: "ahorro-general",
        name: "Ahorro general",
        sort_order: 14,
    },
    StarterCategory {
        slug: "otros",
        name: "Otros",
        sort_order: 15,
    },
];

pub async fn bootstrap(pool: &PgPool) -> Result<Uuid, BootstrapError> {
    let mut transaction = pool.begin().await.map_err(|_| BootstrapError)?;
    let candidate_workspace_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO workspaces (id, singleton_key) VALUES ($1, 'owner') \
         ON CONFLICT (singleton_key) DO NOTHING",
    )
    .bind(candidate_workspace_id)
    .execute(&mut *transaction)
    .await
    .map_err(|_| BootstrapError)?;

    let workspace_id: Uuid =
        sqlx::query_scalar("SELECT id FROM workspaces WHERE singleton_key = 'owner'")
            .fetch_one(&mut *transaction)
            .await
            .map_err(|_| BootstrapError)?;

    for (slug, name, order) in [
        ("general", "General", 1_i16),
        ("transport", "Transporte", 2),
        ("savings", "Ahorro", 3),
    ] {
        sqlx::query("INSERT INTO budget_category_groups (id,workspace_id,slug,name,sort_order) VALUES ($1,$2,$3,$4,$5) ON CONFLICT (workspace_id,slug) DO NOTHING")
            .bind(Uuid::new_v4()).bind(workspace_id).bind(slug).bind(name).bind(order)
            .execute(&mut *transaction).await.map_err(|_| BootstrapError)?;
    }
    let seeded: bool = sqlx::query_scalar(
        "SELECT budget_categories_seeded FROM workspaces WHERE id=$1 FOR UPDATE",
    )
    .bind(workspace_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|_| BootstrapError)?;
    if !seeded {
        for category in STARTER_CATEGORIES {
            sqlx::query(
            "INSERT INTO expense_categories \
             (id, workspace_id, slug, name, sort_order, group_id) \
             VALUES ($1, $2, $3, $4, $5, (SELECT id FROM budget_category_groups WHERE workspace_id=$2 AND slug=$6)) \
             ON CONFLICT (workspace_id, slug) DO NOTHING",
        )
        .bind(Uuid::new_v4())
        .bind(workspace_id)
        .bind(category.slug)
        .bind(category.name)
        .bind(category.sort_order)
        .bind(match category.slug { "transporte" => "transport", "ahorro-general" => "savings", _ => "general" })
        .execute(&mut *transaction)
        .await
        .map_err(|_| BootstrapError)?;
        }

        sqlx::query("UPDATE workspaces SET budget_categories_seeded=TRUE WHERE id=$1")
            .bind(workspace_id)
            .execute(&mut *transaction)
            .await
            .map_err(|_| BootstrapError)?;
    }
    transaction.commit().await.map_err(|_| BootstrapError)?;
    Ok(workspace_id)
}
