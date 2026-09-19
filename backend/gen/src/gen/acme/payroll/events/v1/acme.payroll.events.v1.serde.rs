// @generated
impl serde::Serialize for ProjectCreated {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.project_id != 0 {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.events.v1.ProjectCreated", len)?;
        if self.project_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("projectId", ToString::to_string(&self.project_id).as_str())?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProjectCreated {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "project_id",
            "projectId",
            "name",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProjectId,
            Name,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "projectId" | "project_id" => Ok(GeneratedField::ProjectId),
                            "name" => Ok(GeneratedField::Name),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProjectCreated;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.events.v1.ProjectCreated")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProjectCreated, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut project_id__ = None;
                let mut name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ProjectId => {
                            if project_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projectId"));
                            }
                            project_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ProjectCreated {
                    project_id: project_id__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.events.v1.ProjectCreated", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StaffRegistered {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.staff_id != 0 {
            len += 1;
        }
        if !self.user_id.is_empty() {
            len += 1;
        }
        if !self.display_name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.events.v1.StaffRegistered", len)?;
        if self.staff_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("staffId", ToString::to_string(&self.staff_id).as_str())?;
        }
        if !self.user_id.is_empty() {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if !self.display_name.is_empty() {
            struct_ser.serialize_field("displayName", &self.display_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StaffRegistered {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "staff_id",
            "staffId",
            "user_id",
            "userId",
            "display_name",
            "displayName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StaffId,
            UserId,
            DisplayName,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "staffId" | "staff_id" => Ok(GeneratedField::StaffId),
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            "displayName" | "display_name" => Ok(GeneratedField::DisplayName),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StaffRegistered;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.events.v1.StaffRegistered")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<StaffRegistered, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut staff_id__ = None;
                let mut user_id__ = None;
                let mut display_name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StaffId => {
                            if staff_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("staffId"));
                            }
                            staff_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DisplayName => {
                            if display_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("displayName"));
                            }
                            display_name__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(StaffRegistered {
                    staff_id: staff_id__.unwrap_or_default(),
                    user_id: user_id__.unwrap_or_default(),
                    display_name: display_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.events.v1.StaffRegistered", FIELDS, GeneratedVisitor)
    }
}
