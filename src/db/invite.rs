use super::schema::registration_keys;

#[derive(Insertable)]
#[table_name = "registration_keys"]
pub struct NewRegistrationKey {
    pub value: String,
    pub inviting_user_id: i32,
}
