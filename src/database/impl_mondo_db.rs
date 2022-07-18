use bcrypt::verify;
use mongodb::bson::oid::ObjectId;
use mongodb::{bson, Database};
use rocket::serde::json::Json;

use crate::constants::{EXPIRATION_REFRESH_TOKEN, EXPIRATION_TOKEN};
use crate::database::connect_to_db::MongoDB;
use crate::database::{FindUserBy, LoginError, RegistrationError};
use crate::helper::{find_user_by_login_and_mail, hash_text};
use crate::models::model_element::Element;
use crate::models::model_user::User;
use crate::models::request::login_request::LoginRequest;
use crate::models::request::model_card::DataElementCard;
use crate::models::request::model_login::DataElementLogin;
use crate::models::request::model_note::DataElementNote;
use crate::models::request::model_personal_info::DataElementPersonal;
use crate::models::request::patch_request::EditUserRequest;
use crate::models::request::registration_request::RegistrationRequest;
use crate::private::{JWT_SECRET, REFRESH_JWT_SECRET};
use crate::routes::authorization::token::create_token::encode_token_and_refresh;

impl MongoDB {
    pub fn new(database: Database) -> Self {
        MongoDB { database }
    }

    pub async fn edit_user(
        &self,
        edit_model: Json<EditUserRequest>,
        user: User,
    ) -> mongodb::error::Result<()> {
        let collection = self.database.collection::<User>("user");
        dbg!(
            collection
                .find_one_and_replace(
                    bson::doc! { "_id": user._id },
                    User {
                        _id: user._id,
                        login: edit_model.login.clone(),
                        password: user.password,
                        mail: edit_model.mail.clone(),
                        first_name: edit_model.first_name.clone(),
                        last_name: edit_model.last_name.clone()
                    },
                    None
                )
                .await?
        );
        Ok(())
    }
    pub async fn delete_user(&self, login: &str) -> mongodb::error::Result<()> {
        let collection = self.database.collection::<User>("user");
        collection
            .delete_one(bson::doc! { "login": login }, None)
            .await?;
        Ok(())
    }
    pub async fn find_user_by(
        &self,
        find_by: &str,
        data_find_in: &str,
    ) -> mongodb::error::Result<Option<User>> {
        let collection_user = self.database.collection::<User>("user");

        Ok(collection_user
            .find_one(bson::doc! { find_by: data_find_in }, None)
            .await?)
    }
    pub async fn find_user_by_id(
        &self,
        data_find_in: ObjectId,
    ) -> mongodb::error::Result<Option<User>> {
        let collection_user = self.database.collection::<User>("user");

        Ok(collection_user
            .find_one(bson::doc! { "_id": data_find_in }, None)
            .await?)
    }

    pub async fn login(
        &self,
        login_request: Json<LoginRequest>,
    ) -> mongodb::error::Result<LoginError> {
        match Self::find_user_by(self, "login", &login_request.login).await {
            Ok(option_user) => match option_user {
                None => Ok(LoginError::WrongLogin),
                Some(user) => match verify(&login_request.password, &user.password) {
                    Ok(true) => {
                        match encode_token_and_refresh(
                            user._id.clone(),
                            JWT_SECRET,
                            REFRESH_JWT_SECRET,
                            EXPIRATION_REFRESH_TOKEN,
                            EXPIRATION_TOKEN,
                        ) {
                            Ok(tokens) => Ok(LoginError::Ok(tokens)),
                            Err(_) => Ok(LoginError::Unknown),
                        }
                    }
                    Ok(false) => Ok(LoginError::WrongPassword),
                    Err(_) => Ok(LoginError::WrongPassword),
                },
            },
            Err(_) => Ok(LoginError::WrongLogin),
        }
    }
    pub async fn registration(
        &self,
        registration_request: Json<RegistrationRequest>,
    ) -> mongodb::error::Result<RegistrationError> {
        let collection_user = self.database.collection::<User>("user");
        match find_user_by_login_and_mail(
            self,
            &registration_request.mail,
            &registration_request.login,
        )
        .await
        {
            FindUserBy::UserNotFound => match hash_text(registration_request.password.clone(), 4) {
                Ok(hash_password) => {
                    let user = User {
                        _id: ObjectId::new(),
                        login: registration_request.login.clone(),
                        password: hash_password,
                        mail: registration_request.mail.to_string(),
                        first_name: registration_request.first_name.clone(),
                        last_name: registration_request.last_name.clone(),
                    };
                    collection_user.insert_one(&user, None).await?;
                    match encode_token_and_refresh(
                        user._id.clone(),
                        JWT_SECRET,
                        REFRESH_JWT_SECRET,
                        EXPIRATION_REFRESH_TOKEN,
                        EXPIRATION_TOKEN,
                    ) {
                        Ok(tokens) => Ok(RegistrationError::Ok(tokens)),
                        Err(_) => Ok(RegistrationError::Unknown),
                    }
                }
                Err(_) => Ok(RegistrationError::WrongPassword),
            },
            FindUserBy::UserFoundByEmail => Ok(RegistrationError::AlreadyRegisteredByEmail),
            FindUserBy::UserFoundByLogin => Ok(RegistrationError::AlreadyRegisteredByLogin),
        }
    }

    pub async fn add_element_login(
        &self,
        element: Json<DataElementLogin>,
        id_user: ObjectId,
    ) -> mongodb::error::Result<()> {
        let collection_element = self.database.collection::<Element>("element");

        let element_login = Element {
            id_user,
            name: element.name.clone(),
            login: Some(element.login.clone()),
            password: Some(element.password.clone()),
            url: element.url.clone(),
            owners_name: None,
            number: None,
            type_card: None,
            month_card: None,
            year_card: None,
            ccv: None,
            first_name: None,
            second_name: None,
            last_name: None,
            company: None,
            mail: None,
            telephone: None,
            address_2: None,
            address_1: None,
            city: None,
            region: None,
            index: None,
            country: None,
            description: element.description.clone(),
            favorite: element.favorite,
        };

        collection_element.insert_one(element_login, None).await?;
        Ok(())
    }
    pub async fn add_element_card(
        &self,
        element: Json<DataElementCard>,
        id_user: ObjectId,
    ) -> mongodb::error::Result<()> {
        let collection_element = self.database.collection::<Element>("element");

        let element_card = Element {
            id_user,
            name: element.name.clone(),
            login: None,
            password: None,
            url: None,
            owners_name: element.owners_name.clone(),
            number: element.number.clone(),
            type_card: element.type_card.clone(),
            month_card: element.month_card.clone(),
            year_card: element.year_card.clone(),
            ccv: element.ccv.clone(),
            first_name: None,
            second_name: None,
            last_name: None,
            company: None,
            mail: None,
            telephone: None,
            address_2: None,
            address_1: None,
            city: None,
            region: None,
            index: None,
            country: None,
            description: element.description.clone(),
            favorite: element.favorite,
        };

        collection_element.insert_one(element_card, None).await?;
        Ok(())
    }
    pub async fn add_element_personal_information(
        &self,
        element: Json<DataElementPersonal>,
        id_user: ObjectId,
    ) -> mongodb::error::Result<()> {
        let collection_element = self.database.collection::<Element>("element");

        let element_personal_info = Element {
            id_user,
            name: element.name.clone(),
            login: element.login.clone(),
            password: None,
            url: None,
            owners_name: None,
            number: None,
            type_card: None,
            month_card: None,
            year_card: None,
            ccv: None,
            first_name: element.first_name.clone(),
            second_name: element.second_name.clone(),
            last_name: element.last_name.clone(),
            company: element.company.clone(),
            mail: element.mail.clone(),
            telephone: element.telephone.clone(),
            address_2: element.address_2.clone(),
            address_1: element.address_1.clone(),
            city: element.city.clone(),
            region: element.region.clone(),
            index: element.index.clone(),
            country: element.country.clone(),
            description: element.description.clone(),
            favorite: element.favorite,
        };

        collection_element
            .insert_one(element_personal_info, None)
            .await?;
        Ok(())
    }
    pub async fn add_element_note(
        &self,
        element: Json<DataElementNote>,
        id_user: ObjectId,
    ) -> mongodb::error::Result<()> {
        let collection_element = self.database.collection::<Element>("element");

        let element_note = Element {
            id_user,
            name: element.name.clone(),
            login: None,
            password: None,
            url: None,
            owners_name: None,
            number: None,
            type_card: None,
            month_card: None,
            year_card: None,
            ccv: None,
            first_name: None,
            second_name: None,
            last_name: None,
            company: None,
            mail: None,
            telephone: None,
            address_2: None,
            address_1: None,
            city: None,
            region: None,
            index: None,
            country: None,
            description: element.description.clone(),
            favorite: element.favorite,
        };

        collection_element.insert_one(element_note, None).await?;
        Ok(())
    }
}
