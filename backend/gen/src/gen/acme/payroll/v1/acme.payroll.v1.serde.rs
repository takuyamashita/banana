// @generated
impl serde::Serialize for CreatePayslipRequest {
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
        if self.pay_year != 0 {
            len += 1;
        }
        if self.pay_month != 0 {
            len += 1;
        }
        if !self.lines.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.CreatePayslipRequest", len)?;
        if self.staff_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("staffId", ToString::to_string(&self.staff_id).as_str())?;
        }
        if self.pay_year != 0 {
            struct_ser.serialize_field("payYear", &self.pay_year)?;
        }
        if self.pay_month != 0 {
            struct_ser.serialize_field("payMonth", &self.pay_month)?;
        }
        if !self.lines.is_empty() {
            struct_ser.serialize_field("lines", &self.lines)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePayslipRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "staff_id",
            "staffId",
            "pay_year",
            "payYear",
            "pay_month",
            "payMonth",
            "lines",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StaffId,
            PayYear,
            PayMonth,
            Lines,
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
                            "payYear" | "pay_year" => Ok(GeneratedField::PayYear),
                            "payMonth" | "pay_month" => Ok(GeneratedField::PayMonth),
                            "lines" => Ok(GeneratedField::Lines),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePayslipRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.CreatePayslipRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePayslipRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut staff_id__ = None;
                let mut pay_year__ = None;
                let mut pay_month__ = None;
                let mut lines__ = None;
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
                        GeneratedField::PayYear => {
                            if pay_year__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payYear"));
                            }
                            pay_year__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PayMonth => {
                            if pay_month__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payMonth"));
                            }
                            pay_month__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Lines => {
                            if lines__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lines"));
                            }
                            lines__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreatePayslipRequest {
                    staff_id: staff_id__.unwrap_or_default(),
                    pay_year: pay_year__.unwrap_or_default(),
                    pay_month: pay_month__.unwrap_or_default(),
                    lines: lines__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.CreatePayslipRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreatePayslipResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.payslip_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.CreatePayslipResponse", len)?;
        if self.payslip_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("payslipId", ToString::to_string(&self.payslip_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePayslipResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payslip_id",
            "payslipId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PayslipId,
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
                            "payslipId" | "payslip_id" => Ok(GeneratedField::PayslipId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePayslipResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.CreatePayslipResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePayslipResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payslip_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PayslipId => {
                            if payslip_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payslipId"));
                            }
                            payslip_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(CreatePayslipResponse {
                    payslip_id: payslip_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.CreatePayslipResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateProjectRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.CreateProjectRequest", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateProjectRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
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
            type Value = CreateProjectRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.CreateProjectRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateProjectRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateProjectRequest {
                    name: name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.CreateProjectRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateProjectResponse {
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
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.CreateProjectResponse", len)?;
        if self.project_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("projectId", ToString::to_string(&self.project_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateProjectResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "project_id",
            "projectId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProjectId,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateProjectResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.CreateProjectResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateProjectResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut project_id__ = None;
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
                    }
                }
                Ok(CreateProjectResponse {
                    project_id: project_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.CreateProjectResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateStaffRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.email.is_empty() {
            len += 1;
        }
        if !self.display_name.is_empty() {
            len += 1;
        }
        if !self.temporary_password.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.CreateStaffRequest", len)?;
        if !self.email.is_empty() {
            struct_ser.serialize_field("email", &self.email)?;
        }
        if !self.display_name.is_empty() {
            struct_ser.serialize_field("displayName", &self.display_name)?;
        }
        if !self.temporary_password.is_empty() {
            struct_ser.serialize_field("temporaryPassword", &self.temporary_password)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateStaffRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "email",
            "display_name",
            "displayName",
            "temporary_password",
            "temporaryPassword",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Email,
            DisplayName,
            TemporaryPassword,
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
                            "email" => Ok(GeneratedField::Email),
                            "displayName" | "display_name" => Ok(GeneratedField::DisplayName),
                            "temporaryPassword" | "temporary_password" => Ok(GeneratedField::TemporaryPassword),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateStaffRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.CreateStaffRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateStaffRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut email__ = None;
                let mut display_name__ = None;
                let mut temporary_password__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Email => {
                            if email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("email"));
                            }
                            email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DisplayName => {
                            if display_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("displayName"));
                            }
                            display_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TemporaryPassword => {
                            if temporary_password__.is_some() {
                                return Err(serde::de::Error::duplicate_field("temporaryPassword"));
                            }
                            temporary_password__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateStaffRequest {
                    email: email__.unwrap_or_default(),
                    display_name: display_name__.unwrap_or_default(),
                    temporary_password: temporary_password__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.CreateStaffRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateStaffResponse {
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
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.CreateStaffResponse", len)?;
        if self.staff_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("staffId", ToString::to_string(&self.staff_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateStaffResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "staff_id",
            "staffId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StaffId,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateStaffResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.CreateStaffResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateStaffResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut staff_id__ = None;
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
                    }
                }
                Ok(CreateStaffResponse {
                    staff_id: staff_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.CreateStaffResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FinalizePayslipRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.payslip_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.FinalizePayslipRequest", len)?;
        if self.payslip_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("payslipId", ToString::to_string(&self.payslip_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FinalizePayslipRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payslip_id",
            "payslipId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PayslipId,
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
                            "payslipId" | "payslip_id" => Ok(GeneratedField::PayslipId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FinalizePayslipRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.FinalizePayslipRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FinalizePayslipRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payslip_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PayslipId => {
                            if payslip_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payslipId"));
                            }
                            payslip_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(FinalizePayslipRequest {
                    payslip_id: payslip_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.FinalizePayslipRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FinalizePayslipResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("acme.payroll.v1.FinalizePayslipResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FinalizePayslipResponse {
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
            type Value = FinalizePayslipResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.FinalizePayslipResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FinalizePayslipResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(FinalizePayslipResponse {
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.FinalizePayslipResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetMeRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("acme.payroll.v1.GetMeRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetMeRequest {
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
            type Value = GetMeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.GetMeRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetMeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GetMeRequest {
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.GetMeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetMeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.staff.is_some() {
            len += 1;
        }
        if !self.roles.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.GetMeResponse", len)?;
        if let Some(v) = self.staff.as_ref() {
            struct_ser.serialize_field("staff", v)?;
        }
        if !self.roles.is_empty() {
            struct_ser.serialize_field("roles", &self.roles)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetMeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "staff",
            "roles",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Staff,
            Roles,
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
                            "staff" => Ok(GeneratedField::Staff),
                            "roles" => Ok(GeneratedField::Roles),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetMeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.GetMeResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetMeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut staff__ = None;
                let mut roles__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Staff => {
                            if staff__.is_some() {
                                return Err(serde::de::Error::duplicate_field("staff"));
                            }
                            staff__ = map_.next_value()?;
                        }
                        GeneratedField::Roles => {
                            if roles__.is_some() {
                                return Err(serde::de::Error::duplicate_field("roles"));
                            }
                            roles__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(GetMeResponse {
                    staff: staff__,
                    roles: roles__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.GetMeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetPayslipRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.payslip_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.GetPayslipRequest", len)?;
        if self.payslip_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("payslipId", ToString::to_string(&self.payslip_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetPayslipRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payslip_id",
            "payslipId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PayslipId,
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
                            "payslipId" | "payslip_id" => Ok(GeneratedField::PayslipId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetPayslipRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.GetPayslipRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetPayslipRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payslip_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PayslipId => {
                            if payslip_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payslipId"));
                            }
                            payslip_id__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(GetPayslipRequest {
                    payslip_id: payslip_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.GetPayslipRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetPayslipResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.payslip.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.GetPayslipResponse", len)?;
        if let Some(v) = self.payslip.as_ref() {
            struct_ser.serialize_field("payslip", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetPayslipResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payslip",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Payslip,
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
                            "payslip" => Ok(GeneratedField::Payslip),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetPayslipResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.GetPayslipResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetPayslipResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payslip__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Payslip => {
                            if payslip__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payslip"));
                            }
                            payslip__ = map_.next_value()?;
                        }
                    }
                }
                Ok(GetPayslipResponse {
                    payslip: payslip__,
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.GetPayslipResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListPayslipsRequest {
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
        if self.page_size != 0 {
            len += 1;
        }
        if !self.page_token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.ListPayslipsRequest", len)?;
        if self.staff_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("staffId", ToString::to_string(&self.staff_id).as_str())?;
        }
        if self.page_size != 0 {
            struct_ser.serialize_field("pageSize", &self.page_size)?;
        }
        if !self.page_token.is_empty() {
            struct_ser.serialize_field("pageToken", &self.page_token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListPayslipsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "staff_id",
            "staffId",
            "page_size",
            "pageSize",
            "page_token",
            "pageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StaffId,
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
                            "staffId" | "staff_id" => Ok(GeneratedField::StaffId),
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
            type Value = ListPayslipsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.ListPayslipsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListPayslipsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut staff_id__ = None;
                let mut page_size__ = None;
                let mut page_token__ = None;
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
                Ok(ListPayslipsRequest {
                    staff_id: staff_id__.unwrap_or_default(),
                    page_size: page_size__.unwrap_or_default(),
                    page_token: page_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.ListPayslipsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListPayslipsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.payslips.is_empty() {
            len += 1;
        }
        if !self.next_page_token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.ListPayslipsResponse", len)?;
        if !self.payslips.is_empty() {
            struct_ser.serialize_field("payslips", &self.payslips)?;
        }
        if !self.next_page_token.is_empty() {
            struct_ser.serialize_field("nextPageToken", &self.next_page_token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListPayslipsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payslips",
            "next_page_token",
            "nextPageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Payslips,
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
                            "payslips" => Ok(GeneratedField::Payslips),
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
            type Value = ListPayslipsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.ListPayslipsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListPayslipsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payslips__ = None;
                let mut next_page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Payslips => {
                            if payslips__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payslips"));
                            }
                            payslips__ = Some(map_.next_value()?);
                        }
                        GeneratedField::NextPageToken => {
                            if next_page_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextPageToken"));
                            }
                            next_page_token__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ListPayslipsResponse {
                    payslips: payslips__.unwrap_or_default(),
                    next_page_token: next_page_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.ListPayslipsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListProjectsRequest {
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
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.ListProjectsRequest", len)?;
        if self.page_size != 0 {
            struct_ser.serialize_field("pageSize", &self.page_size)?;
        }
        if !self.page_token.is_empty() {
            struct_ser.serialize_field("pageToken", &self.page_token)?;
        }
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
            type Value = ListProjectsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.ListProjectsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListProjectsRequest, V::Error>
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
                Ok(ListProjectsRequest {
                    page_size: page_size__.unwrap_or_default(),
                    page_token: page_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.ListProjectsRequest", FIELDS, GeneratedVisitor)
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
        if !self.next_page_token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.ListProjectsResponse", len)?;
        if !self.projects.is_empty() {
            struct_ser.serialize_field("projects", &self.projects)?;
        }
        if !self.next_page_token.is_empty() {
            struct_ser.serialize_field("nextPageToken", &self.next_page_token)?;
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
            "next_page_token",
            "nextPageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Projects,
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
                            "projects" => Ok(GeneratedField::Projects),
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
            type Value = ListProjectsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.ListProjectsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListProjectsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut projects__ = None;
                let mut next_page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Projects => {
                            if projects__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projects"));
                            }
                            projects__ = Some(map_.next_value()?);
                        }
                        GeneratedField::NextPageToken => {
                            if next_page_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextPageToken"));
                            }
                            next_page_token__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ListProjectsResponse {
                    projects: projects__.unwrap_or_default(),
                    next_page_token: next_page_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.ListProjectsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListStaffRequest {
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
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.ListStaffRequest", len)?;
        if self.page_size != 0 {
            struct_ser.serialize_field("pageSize", &self.page_size)?;
        }
        if !self.page_token.is_empty() {
            struct_ser.serialize_field("pageToken", &self.page_token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListStaffRequest {
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
            type Value = ListStaffRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.ListStaffRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListStaffRequest, V::Error>
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
                Ok(ListStaffRequest {
                    page_size: page_size__.unwrap_or_default(),
                    page_token: page_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.ListStaffRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListStaffResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.staff.is_empty() {
            len += 1;
        }
        if !self.next_page_token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.ListStaffResponse", len)?;
        if !self.staff.is_empty() {
            struct_ser.serialize_field("staff", &self.staff)?;
        }
        if !self.next_page_token.is_empty() {
            struct_ser.serialize_field("nextPageToken", &self.next_page_token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListStaffResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "staff",
            "next_page_token",
            "nextPageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Staff,
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
                            "staff" => Ok(GeneratedField::Staff),
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
            type Value = ListStaffResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.ListStaffResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListStaffResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut staff__ = None;
                let mut next_page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Staff => {
                            if staff__.is_some() {
                                return Err(serde::de::Error::duplicate_field("staff"));
                            }
                            staff__ = Some(map_.next_value()?);
                        }
                        GeneratedField::NextPageToken => {
                            if next_page_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextPageToken"));
                            }
                            next_page_token__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ListStaffResponse {
                    staff: staff__.unwrap_or_default(),
                    next_page_token: next_page_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.ListStaffResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Payslip {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.payslip_id != 0 {
            len += 1;
        }
        if self.staff_id != 0 {
            len += 1;
        }
        if self.pay_year != 0 {
            len += 1;
        }
        if self.pay_month != 0 {
            len += 1;
        }
        if self.status != 0 {
            len += 1;
        }
        if self.total_yen != 0 {
            len += 1;
        }
        if !self.lines.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.Payslip", len)?;
        if self.payslip_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("payslipId", ToString::to_string(&self.payslip_id).as_str())?;
        }
        if self.staff_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("staffId", ToString::to_string(&self.staff_id).as_str())?;
        }
        if self.pay_year != 0 {
            struct_ser.serialize_field("payYear", &self.pay_year)?;
        }
        if self.pay_month != 0 {
            struct_ser.serialize_field("payMonth", &self.pay_month)?;
        }
        if self.status != 0 {
            let v = PayslipStatus::try_from(self.status)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.status)))?;
            struct_ser.serialize_field("status", &v)?;
        }
        if self.total_yen != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("totalYen", ToString::to_string(&self.total_yen).as_str())?;
        }
        if !self.lines.is_empty() {
            struct_ser.serialize_field("lines", &self.lines)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Payslip {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payslip_id",
            "payslipId",
            "staff_id",
            "staffId",
            "pay_year",
            "payYear",
            "pay_month",
            "payMonth",
            "status",
            "total_yen",
            "totalYen",
            "lines",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PayslipId,
            StaffId,
            PayYear,
            PayMonth,
            Status,
            TotalYen,
            Lines,
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
                            "payslipId" | "payslip_id" => Ok(GeneratedField::PayslipId),
                            "staffId" | "staff_id" => Ok(GeneratedField::StaffId),
                            "payYear" | "pay_year" => Ok(GeneratedField::PayYear),
                            "payMonth" | "pay_month" => Ok(GeneratedField::PayMonth),
                            "status" => Ok(GeneratedField::Status),
                            "totalYen" | "total_yen" => Ok(GeneratedField::TotalYen),
                            "lines" => Ok(GeneratedField::Lines),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Payslip;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.Payslip")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Payslip, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payslip_id__ = None;
                let mut staff_id__ = None;
                let mut pay_year__ = None;
                let mut pay_month__ = None;
                let mut status__ = None;
                let mut total_yen__ = None;
                let mut lines__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PayslipId => {
                            if payslip_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payslipId"));
                            }
                            payslip_id__ = 
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
                        GeneratedField::PayYear => {
                            if pay_year__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payYear"));
                            }
                            pay_year__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PayMonth => {
                            if pay_month__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payMonth"));
                            }
                            pay_month__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = Some(map_.next_value::<PayslipStatus>()? as i32);
                        }
                        GeneratedField::TotalYen => {
                            if total_yen__.is_some() {
                                return Err(serde::de::Error::duplicate_field("totalYen"));
                            }
                            total_yen__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Lines => {
                            if lines__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lines"));
                            }
                            lines__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Payslip {
                    payslip_id: payslip_id__.unwrap_or_default(),
                    staff_id: staff_id__.unwrap_or_default(),
                    pay_year: pay_year__.unwrap_or_default(),
                    pay_month: pay_month__.unwrap_or_default(),
                    status: status__.unwrap_or_default(),
                    total_yen: total_yen__.unwrap_or_default(),
                    lines: lines__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.Payslip", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PayslipLine {
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
        if self.hourly_rate != 0 {
            len += 1;
        }
        if self.amount_yen != 0 {
            len += 1;
        }
        if !self.project_name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.PayslipLine", len)?;
        if self.project_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("projectId", ToString::to_string(&self.project_id).as_str())?;
        }
        if self.work_minutes != 0 {
            struct_ser.serialize_field("workMinutes", &self.work_minutes)?;
        }
        if self.hourly_rate != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("hourlyRate", ToString::to_string(&self.hourly_rate).as_str())?;
        }
        if self.amount_yen != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("amountYen", ToString::to_string(&self.amount_yen).as_str())?;
        }
        if !self.project_name.is_empty() {
            struct_ser.serialize_field("projectName", &self.project_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PayslipLine {
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
            "hourly_rate",
            "hourlyRate",
            "amount_yen",
            "amountYen",
            "project_name",
            "projectName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProjectId,
            WorkMinutes,
            HourlyRate,
            AmountYen,
            ProjectName,
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
                            "hourlyRate" | "hourly_rate" => Ok(GeneratedField::HourlyRate),
                            "amountYen" | "amount_yen" => Ok(GeneratedField::AmountYen),
                            "projectName" | "project_name" => Ok(GeneratedField::ProjectName),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PayslipLine;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.PayslipLine")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PayslipLine, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut project_id__ = None;
                let mut work_minutes__ = None;
                let mut hourly_rate__ = None;
                let mut amount_yen__ = None;
                let mut project_name__ = None;
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
                        GeneratedField::HourlyRate => {
                            if hourly_rate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hourlyRate"));
                            }
                            hourly_rate__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AmountYen => {
                            if amount_yen__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amountYen"));
                            }
                            amount_yen__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProjectName => {
                            if project_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("projectName"));
                            }
                            project_name__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(PayslipLine {
                    project_id: project_id__.unwrap_or_default(),
                    work_minutes: work_minutes__.unwrap_or_default(),
                    hourly_rate: hourly_rate__.unwrap_or_default(),
                    amount_yen: amount_yen__.unwrap_or_default(),
                    project_name: project_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.PayslipLine", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PayslipLineInput {
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
        if self.hourly_rate != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.PayslipLineInput", len)?;
        if self.project_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("projectId", ToString::to_string(&self.project_id).as_str())?;
        }
        if self.work_minutes != 0 {
            struct_ser.serialize_field("workMinutes", &self.work_minutes)?;
        }
        if self.hourly_rate != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("hourlyRate", ToString::to_string(&self.hourly_rate).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PayslipLineInput {
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
            "hourly_rate",
            "hourlyRate",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProjectId,
            WorkMinutes,
            HourlyRate,
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
                            "hourlyRate" | "hourly_rate" => Ok(GeneratedField::HourlyRate),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PayslipLineInput;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.PayslipLineInput")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PayslipLineInput, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut project_id__ = None;
                let mut work_minutes__ = None;
                let mut hourly_rate__ = None;
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
                        GeneratedField::HourlyRate => {
                            if hourly_rate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hourlyRate"));
                            }
                            hourly_rate__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(PayslipLineInput {
                    project_id: project_id__.unwrap_or_default(),
                    work_minutes: work_minutes__.unwrap_or_default(),
                    hourly_rate: hourly_rate__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.PayslipLineInput", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PayslipStatus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "PAYSLIP_STATUS_UNSPECIFIED",
            Self::Draft => "PAYSLIP_STATUS_DRAFT",
            Self::Finalized => "PAYSLIP_STATUS_FINALIZED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for PayslipStatus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "PAYSLIP_STATUS_UNSPECIFIED",
            "PAYSLIP_STATUS_DRAFT",
            "PAYSLIP_STATUS_FINALIZED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PayslipStatus;

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
                    "PAYSLIP_STATUS_UNSPECIFIED" => Ok(PayslipStatus::Unspecified),
                    "PAYSLIP_STATUS_DRAFT" => Ok(PayslipStatus::Draft),
                    "PAYSLIP_STATUS_FINALIZED" => Ok(PayslipStatus::Finalized),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.Project", len)?;
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
                formatter.write_str("struct acme.payroll.v1.Project")
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
        deserializer.deserialize_struct("acme.payroll.v1.Project", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Staff {
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
        if !self.email.is_empty() {
            len += 1;
        }
        if !self.display_name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("acme.payroll.v1.Staff", len)?;
        if self.staff_id != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("staffId", ToString::to_string(&self.staff_id).as_str())?;
        }
        if !self.email.is_empty() {
            struct_ser.serialize_field("email", &self.email)?;
        }
        if !self.display_name.is_empty() {
            struct_ser.serialize_field("displayName", &self.display_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Staff {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "staff_id",
            "staffId",
            "email",
            "display_name",
            "displayName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StaffId,
            Email,
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
                            "email" => Ok(GeneratedField::Email),
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
            type Value = Staff;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct acme.payroll.v1.Staff")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Staff, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut staff_id__ = None;
                let mut email__ = None;
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
                        GeneratedField::Email => {
                            if email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("email"));
                            }
                            email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DisplayName => {
                            if display_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("displayName"));
                            }
                            display_name__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Staff {
                    staff_id: staff_id__.unwrap_or_default(),
                    email: email__.unwrap_or_default(),
                    display_name: display_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("acme.payroll.v1.Staff", FIELDS, GeneratedVisitor)
    }
}
