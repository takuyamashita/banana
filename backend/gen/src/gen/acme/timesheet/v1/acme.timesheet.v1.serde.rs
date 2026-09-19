// @generated
impl serde::Serialize for ApproveTimesheetRequest {
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
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.ApproveTimesheetRequest", len)?;
        if self.timesheet_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("timesheetId", ToString::to_string(&self.timesheet_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ApproveTimesheetRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timesheet_id",
            "timesheetId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TimesheetId,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ApproveTimesheetRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.ApproveTimesheetRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ApproveTimesheetRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timesheet_id__ = None;
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
                    }
                }
                Ok(ApproveTimesheetRequest {
                    timesheet_id: timesheet_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.ApproveTimesheetRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ApproveTimesheetResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.timesheet.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.ApproveTimesheetResponse", len)?;
        if let Some(v) = self.timesheet.as_ref() {
            struct_ser.serialize_field("timesheet", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ApproveTimesheetResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timesheet",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Timesheet,
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
                            "timesheet" => Ok(GeneratedField::Timesheet),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ApproveTimesheetResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.ApproveTimesheetResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ApproveTimesheetResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timesheet__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Timesheet => {
                            if timesheet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timesheet"));
                            }
                            timesheet__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ApproveTimesheetResponse {
                    timesheet: timesheet__,
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.ApproveTimesheetResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetMyTimesheetRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.year != 0 {
            len += 1;
        }
        if self.month != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.GetMyTimesheetRequest", len)?;
        if self.year != 0 {
            struct_ser.serialize_field("year", &self.year)?;
        }
        if self.month != 0 {
            struct_ser.serialize_field("month", &self.month)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetMyTimesheetRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "year",
            "month",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Year,
            Month,
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
                            "year" => Ok(GeneratedField::Year),
                            "month" => Ok(GeneratedField::Month),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetMyTimesheetRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.GetMyTimesheetRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetMyTimesheetRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut year__ = None;
                let mut month__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                    }
                }
                Ok(GetMyTimesheetRequest {
                    year: year__.unwrap_or_default(),
                    month: month__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.GetMyTimesheetRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetMyTimesheetResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.timesheet.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.GetMyTimesheetResponse", len)?;
        if let Some(v) = self.timesheet.as_ref() {
            struct_ser.serialize_field("timesheet", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetMyTimesheetResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timesheet",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Timesheet,
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
                            "timesheet" => Ok(GeneratedField::Timesheet),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetMyTimesheetResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.GetMyTimesheetResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetMyTimesheetResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timesheet__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Timesheet => {
                            if timesheet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timesheet"));
                            }
                            timesheet__ = map_.next_value()?;
                        }
                    }
                }
                Ok(GetMyTimesheetResponse {
                    timesheet: timesheet__,
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.GetMyTimesheetResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListProjectsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("acme.timesheet.v1.ListProjectsRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListProjectsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListProjectsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.ListProjectsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListProjectsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ListProjectsRequest {
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.ListProjectsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListProjectsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.projects.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.ListProjectsResponse", len)?;
        if !self.projects.is_empty() {
            struct_ser.serialize_field("projects", &self.projects)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListProjectsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "projects",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Projects,
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
                            "projects" => Ok(GeneratedField::Projects),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListProjectsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.ListProjectsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListProjectsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut projects__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Projects => {
                            if projects__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projects"));
                            }
                            projects__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ListProjectsResponse {
                    projects: projects__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.ListProjectsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListSubmittedTimesheetsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.page_size != 0 {
            len += 1;
        }
        if !self.page_token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.ListSubmittedTimesheetsRequest", len)?;
        if self.page_size != 0 {
            struct_ser.serialize_field("pageSize", &self.page_size)?;
        }
        if !self.page_token.is_empty() {
            struct_ser.serialize_field("pageToken", &self.page_token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListSubmittedTimesheetsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "page_size",
            "pageSize",
            "page_token",
            "pageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PageSize,
            PageToken,
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
                            "pageSize" | "page_size" => Ok(GeneratedField::PageSize),
                            "pageToken" | "page_token" => Ok(GeneratedField::PageToken),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListSubmittedTimesheetsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.ListSubmittedTimesheetsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListSubmittedTimesheetsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut page_size__ = None;
                let mut page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PageSize => {
                            if page_size__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pageSize"));
                            }
                            page_size__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PageToken => {
                            if page_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pageToken"));
                            }
                            page_token__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ListSubmittedTimesheetsRequest {
                    page_size: page_size__.unwrap_or_default(),
                    page_token: page_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.ListSubmittedTimesheetsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListSubmittedTimesheetsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.timesheets.is_empty() {
            len += 1;
        }
        if !self.next_page_token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.ListSubmittedTimesheetsResponse", len)?;
        if !self.timesheets.is_empty() {
            struct_ser.serialize_field("timesheets", &self.timesheets)?;
        }
        if !self.next_page_token.is_empty() {
            struct_ser.serialize_field("nextPageToken", &self.next_page_token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListSubmittedTimesheetsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timesheets",
            "next_page_token",
            "nextPageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Timesheets,
            NextPageToken,
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
                            "timesheets" => Ok(GeneratedField::Timesheets),
                            "nextPageToken" | "next_page_token" => Ok(GeneratedField::NextPageToken),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListSubmittedTimesheetsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.ListSubmittedTimesheetsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListSubmittedTimesheetsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timesheets__ = None;
                let mut next_page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Timesheets => {
                            if timesheets__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timesheets"));
                            }
                            timesheets__ = Some(map_.next_value()?);
                        }
                        GeneratedField::NextPageToken => {
                            if next_page_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextPageToken"));
                            }
                            next_page_token__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ListSubmittedTimesheetsResponse {
                    timesheets: timesheets__.unwrap_or_default(),
                    next_page_token: next_page_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.ListSubmittedTimesheetsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Project {
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
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.Project", len)?;
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
impl<'de> serde::Deserialize<'de> for Project {
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
            type Value = Project;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.Project")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Project, V::Error>
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
                Ok(Project {
                    project_id: project_id__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.Project", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ReturnTimesheetRequest {
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
        if !self.reason.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.ReturnTimesheetRequest", len)?;
        if self.timesheet_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("timesheetId", ToString::to_string(&self.timesheet_id).as_str())?;
        }
        if !self.reason.is_empty() {
            struct_ser.serialize_field("reason", &self.reason)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ReturnTimesheetRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timesheet_id",
            "timesheetId",
            "reason",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TimesheetId,
            Reason,
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
                            "reason" => Ok(GeneratedField::Reason),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ReturnTimesheetRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.ReturnTimesheetRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ReturnTimesheetRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timesheet_id__ = None;
                let mut reason__ = None;
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
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ReturnTimesheetRequest {
                    timesheet_id: timesheet_id__.unwrap_or_default(),
                    reason: reason__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.ReturnTimesheetRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ReturnTimesheetResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.timesheet.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.ReturnTimesheetResponse", len)?;
        if let Some(v) = self.timesheet.as_ref() {
            struct_ser.serialize_field("timesheet", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ReturnTimesheetResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timesheet",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Timesheet,
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
                            "timesheet" => Ok(GeneratedField::Timesheet),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ReturnTimesheetResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.ReturnTimesheetResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ReturnTimesheetResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timesheet__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Timesheet => {
                            if timesheet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timesheet"));
                            }
                            timesheet__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ReturnTimesheetResponse {
                    timesheet: timesheet__,
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.ReturnTimesheetResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SaveMyTimesheetRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.year != 0 {
            len += 1;
        }
        if self.month != 0 {
            len += 1;
        }
        if !self.entries.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.SaveMyTimesheetRequest", len)?;
        if self.year != 0 {
            struct_ser.serialize_field("year", &self.year)?;
        }
        if self.month != 0 {
            struct_ser.serialize_field("month", &self.month)?;
        }
        if !self.entries.is_empty() {
            struct_ser.serialize_field("entries", &self.entries)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SaveMyTimesheetRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "year",
            "month",
            "entries",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Year,
            Month,
            Entries,
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
                            "year" => Ok(GeneratedField::Year),
                            "month" => Ok(GeneratedField::Month),
                            "entries" => Ok(GeneratedField::Entries),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SaveMyTimesheetRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.SaveMyTimesheetRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SaveMyTimesheetRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut year__ = None;
                let mut month__ = None;
                let mut entries__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                        GeneratedField::Entries => {
                            if entries__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entries"));
                            }
                            entries__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SaveMyTimesheetRequest {
                    year: year__.unwrap_or_default(),
                    month: month__.unwrap_or_default(),
                    entries: entries__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.SaveMyTimesheetRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SaveMyTimesheetResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.timesheet.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.SaveMyTimesheetResponse", len)?;
        if let Some(v) = self.timesheet.as_ref() {
            struct_ser.serialize_field("timesheet", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SaveMyTimesheetResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timesheet",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Timesheet,
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
                            "timesheet" => Ok(GeneratedField::Timesheet),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SaveMyTimesheetResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.SaveMyTimesheetResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SaveMyTimesheetResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timesheet__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Timesheet => {
                            if timesheet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timesheet"));
                            }
                            timesheet__ = map_.next_value()?;
                        }
                    }
                }
                Ok(SaveMyTimesheetResponse {
                    timesheet: timesheet__,
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.SaveMyTimesheetResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SubmitMyTimesheetRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.year != 0 {
            len += 1;
        }
        if self.month != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.SubmitMyTimesheetRequest", len)?;
        if self.year != 0 {
            struct_ser.serialize_field("year", &self.year)?;
        }
        if self.month != 0 {
            struct_ser.serialize_field("month", &self.month)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SubmitMyTimesheetRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "year",
            "month",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Year,
            Month,
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
                            "year" => Ok(GeneratedField::Year),
                            "month" => Ok(GeneratedField::Month),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SubmitMyTimesheetRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.SubmitMyTimesheetRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SubmitMyTimesheetRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut year__ = None;
                let mut month__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                    }
                }
                Ok(SubmitMyTimesheetRequest {
                    year: year__.unwrap_or_default(),
                    month: month__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.SubmitMyTimesheetRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SubmitMyTimesheetResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.timesheet.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.SubmitMyTimesheetResponse", len)?;
        if let Some(v) = self.timesheet.as_ref() {
            struct_ser.serialize_field("timesheet", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SubmitMyTimesheetResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timesheet",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Timesheet,
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
                            "timesheet" => Ok(GeneratedField::Timesheet),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SubmitMyTimesheetResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.SubmitMyTimesheetResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SubmitMyTimesheetResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timesheet__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Timesheet => {
                            if timesheet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timesheet"));
                            }
                            timesheet__ = map_.next_value()?;
                        }
                    }
                }
                Ok(SubmitMyTimesheetResponse {
                    timesheet: timesheet__,
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.SubmitMyTimesheetResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Timesheet {
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
        if !self.staff_name.is_empty() {
            len += 1;
        }
        if self.year != 0 {
            len += 1;
        }
        if self.month != 0 {
            len += 1;
        }
        if self.status != 0 {
            len += 1;
        }
        if !self.entries.is_empty() {
            len += 1;
        }
        if self.total_minutes != 0 {
            len += 1;
        }
        if !self.returned_reason.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.Timesheet", len)?;
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
        if !self.staff_name.is_empty() {
            struct_ser.serialize_field("staffName", &self.staff_name)?;
        }
        if self.year != 0 {
            struct_ser.serialize_field("year", &self.year)?;
        }
        if self.month != 0 {
            struct_ser.serialize_field("month", &self.month)?;
        }
        if self.status != 0 {
            let v = TimesheetStatus::try_from(self.status)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.status)))?;
            struct_ser.serialize_field("status", &v)?;
        }
        if !self.entries.is_empty() {
            struct_ser.serialize_field("entries", &self.entries)?;
        }
        if self.total_minutes != 0 {
            struct_ser.serialize_field("totalMinutes", &self.total_minutes)?;
        }
        if !self.returned_reason.is_empty() {
            struct_ser.serialize_field("returnedReason", &self.returned_reason)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Timesheet {
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
            "staff_name",
            "staffName",
            "year",
            "month",
            "status",
            "entries",
            "total_minutes",
            "totalMinutes",
            "returned_reason",
            "returnedReason",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TimesheetId,
            StaffId,
            StaffName,
            Year,
            Month,
            Status,
            Entries,
            TotalMinutes,
            ReturnedReason,
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
                            "staffName" | "staff_name" => Ok(GeneratedField::StaffName),
                            "year" => Ok(GeneratedField::Year),
                            "month" => Ok(GeneratedField::Month),
                            "status" => Ok(GeneratedField::Status),
                            "entries" => Ok(GeneratedField::Entries),
                            "totalMinutes" | "total_minutes" => Ok(GeneratedField::TotalMinutes),
                            "returnedReason" | "returned_reason" => Ok(GeneratedField::ReturnedReason),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Timesheet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.Timesheet")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Timesheet, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timesheet_id__ = None;
                let mut staff_id__ = None;
                let mut staff_name__ = None;
                let mut year__ = None;
                let mut month__ = None;
                let mut status__ = None;
                let mut entries__ = None;
                let mut total_minutes__ = None;
                let mut returned_reason__ = None;
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
                        GeneratedField::StaffName => {
                            if staff_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("staffName"));
                            }
                            staff_name__ = Some(map_.next_value()?);
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
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = Some(map_.next_value::<TimesheetStatus>()? as i32);
                        }
                        GeneratedField::Entries => {
                            if entries__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entries"));
                            }
                            entries__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TotalMinutes => {
                            if total_minutes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("totalMinutes"));
                            }
                            total_minutes__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ReturnedReason => {
                            if returned_reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("returnedReason"));
                            }
                            returned_reason__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Timesheet {
                    timesheet_id: timesheet_id__.unwrap_or_default(),
                    staff_id: staff_id__.unwrap_or_default(),
                    staff_name: staff_name__.unwrap_or_default(),
                    year: year__.unwrap_or_default(),
                    month: month__.unwrap_or_default(),
                    status: status__.unwrap_or_default(),
                    entries: entries__.unwrap_or_default(),
                    total_minutes: total_minutes__.unwrap_or_default(),
                    returned_reason: returned_reason__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.Timesheet", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TimesheetStatus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "TIMESHEET_STATUS_UNSPECIFIED",
            Self::Draft => "TIMESHEET_STATUS_DRAFT",
            Self::Submitted => "TIMESHEET_STATUS_SUBMITTED",
            Self::Approved => "TIMESHEET_STATUS_APPROVED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for TimesheetStatus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "TIMESHEET_STATUS_UNSPECIFIED",
            "TIMESHEET_STATUS_DRAFT",
            "TIMESHEET_STATUS_SUBMITTED",
            "TIMESHEET_STATUS_APPROVED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TimesheetStatus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "TIMESHEET_STATUS_UNSPECIFIED" => Ok(TimesheetStatus::Unspecified),
                    "TIMESHEET_STATUS_DRAFT" => Ok(TimesheetStatus::Draft),
                    "TIMESHEET_STATUS_SUBMITTED" => Ok(TimesheetStatus::Submitted),
                    "TIMESHEET_STATUS_APPROVED" => Ok(TimesheetStatus::Approved),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for WorkEntry {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.date.is_empty() {
            len += 1;
        }
        if self.project_id != 0 {
            len += 1;
        }
        if !self.project_name.is_empty() {
            len += 1;
        }
        if self.work_minutes != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.WorkEntry", len)?;
        if !self.date.is_empty() {
            struct_ser.serialize_field("date", &self.date)?;
        }
        if self.project_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("projectId", ToString::to_string(&self.project_id).as_str())?;
        }
        if !self.project_name.is_empty() {
            struct_ser.serialize_field("projectName", &self.project_name)?;
        }
        if self.work_minutes != 0 {
            struct_ser.serialize_field("workMinutes", &self.work_minutes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WorkEntry {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "date",
            "project_id",
            "projectId",
            "project_name",
            "projectName",
            "work_minutes",
            "workMinutes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Date,
            ProjectId,
            ProjectName,
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
                            "date" => Ok(GeneratedField::Date),
                            "projectId" | "project_id" => Ok(GeneratedField::ProjectId),
                            "projectName" | "project_name" => Ok(GeneratedField::ProjectName),
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
            type Value = WorkEntry;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.WorkEntry")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WorkEntry, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut date__ = None;
                let mut project_id__ = None;
                let mut project_name__ = None;
                let mut work_minutes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Date => {
                            if date__.is_some() {
                                return Err(serde::de::Error::duplicate_field("date"));
                            }
                            date__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ProjectId => {
                            if project_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projectId"));
                            }
                            project_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProjectName => {
                            if project_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projectName"));
                            }
                            project_name__ = Some(map_.next_value()?);
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
                Ok(WorkEntry {
                    date: date__.unwrap_or_default(),
                    project_id: project_id__.unwrap_or_default(),
                    project_name: project_name__.unwrap_or_default(),
                    work_minutes: work_minutes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.WorkEntry", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WorkEntryInput {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.date.is_empty() {
            len += 1;
        }
        if self.project_id != 0 {
            len += 1;
        }
        if self.work_minutes != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.timesheet.v1.WorkEntryInput", len)?;
        if !self.date.is_empty() {
            struct_ser.serialize_field("date", &self.date)?;
        }
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
impl<'de> serde::Deserialize<'de> for WorkEntryInput {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "date",
            "project_id",
            "projectId",
            "work_minutes",
            "workMinutes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Date,
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
                            "date" => Ok(GeneratedField::Date),
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
            type Value = WorkEntryInput;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.timesheet.v1.WorkEntryInput")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WorkEntryInput, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut date__ = None;
                let mut project_id__ = None;
                let mut work_minutes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Date => {
                            if date__.is_some() {
                                return Err(serde::de::Error::duplicate_field("date"));
                            }
                            date__ = Some(map_.next_value()?);
                        }
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
                Ok(WorkEntryInput {
                    date: date__.unwrap_or_default(),
                    project_id: project_id__.unwrap_or_default(),
                    work_minutes: work_minutes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.timesheet.v1.WorkEntryInput", FIELDS, GeneratedVisitor)
    }
}
