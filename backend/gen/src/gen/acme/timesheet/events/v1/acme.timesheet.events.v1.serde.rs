// @generated
impl serde::Serialize for ProjectWork {
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
        if self.work_minutes != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.events.v1.ProjectWork", len)?;
        if self.project_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("projectId", ToString::to_string(&self.project_id).as_str())?;
        }
        if self.work_minutes != 0 {
            struct_ser.serialize_field("workMinutes", &self.work_minutes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ProjectWork {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "project_id",
            "projectId",
            "work_minutes",
            "workMinutes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProjectId,
            WorkMinutes,
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
                            "workMinutes" | "work_minutes" => Ok(GeneratedField::WorkMinutes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProjectWork;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.events.v1.ProjectWork")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ProjectWork, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut project_id__ = None;
                let mut work_minutes__ = None;
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
                        GeneratedField::WorkMinutes => {
                            if work_minutes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("workMinutes"));
                            }
                            work_minutes__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ProjectWork {
                    project_id: project_id__.unwrap_or_default(),
                    work_minutes: work_minutes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.events.v1.ProjectWork", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TimesheetApproved {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.timesheet_id != 0 {
            len += 1;
        }
        if self.staff_id != 0 {
            len += 1;
        }
        if self.year != 0 {
            len += 1;
        }
        if self.month != 0 {
            len += 1;
        }
        if !self.work.is_empty() {
            len += 1;
        }
        if !self.approved_at.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.events.v1.TimesheetApproved", len)?;
        if self.timesheet_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("timesheetId", ToString::to_string(&self.timesheet_id).as_str())?;
        }
        if self.staff_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("staffId", ToString::to_string(&self.staff_id).as_str())?;
        }
        if self.year != 0 {
            struct_ser.serialize_field("year", &self.year)?;
        }
        if self.month != 0 {
            struct_ser.serialize_field("month", &self.month)?;
        }
        if !self.work.is_empty() {
            struct_ser.serialize_field("work", &self.work)?;
        }
        if !self.approved_at.is_empty() {
            struct_ser.serialize_field("approvedAt", &self.approved_at)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TimesheetApproved {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timesheet_id",
            "timesheetId",
            "staff_id",
            "staffId",
            "year",
            "month",
            "work",
            "approved_at",
            "approvedAt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TimesheetId,
            StaffId,
            Year,
            Month,
            Work,
            ApprovedAt,
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
                            "timesheetId" | "timesheet_id" => Ok(GeneratedField::TimesheetId),
                            "staffId" | "staff_id" => Ok(GeneratedField::StaffId),
                            "year" => Ok(GeneratedField::Year),
                            "month" => Ok(GeneratedField::Month),
                            "work" => Ok(GeneratedField::Work),
                            "approvedAt" | "approved_at" => Ok(GeneratedField::ApprovedAt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TimesheetApproved;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.events.v1.TimesheetApproved")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TimesheetApproved, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timesheet_id__ = None;
                let mut staff_id__ = None;
                let mut year__ = None;
                let mut month__ = None;
                let mut work__ = None;
                let mut approved_at__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TimesheetId => {
                            if timesheet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timesheetId"));
                            }
                            timesheet_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::StaffId => {
                            if staff_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("staffId"));
                            }
                            staff_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Year => {
                            if year__.is_some() {
                                return Err(serde::de::Error::duplicate_field("year"));
                            }
                            year__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Month => {
                            if month__.is_some() {
                                return Err(serde::de::Error::duplicate_field("month"));
                            }
                            month__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Work => {
                            if work__.is_some() {
                                return Err(serde::de::Error::duplicate_field("work"));
                            }
                            work__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ApprovedAt => {
                            if approved_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("approvedAt"));
                            }
                            approved_at__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(TimesheetApproved {
                    timesheet_id: timesheet_id__.unwrap_or_default(),
                    staff_id: staff_id__.unwrap_or_default(),
                    year: year__.unwrap_or_default(),
                    month: month__.unwrap_or_default(),
                    work: work__.unwrap_or_default(),
                    approved_at: approved_at__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.events.v1.TimesheetApproved", FIELDS, GeneratedVisitor)
    }
}
