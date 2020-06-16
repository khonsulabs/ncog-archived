-save-item = Save {$type}
-saved-item = {$type} was saved successfully.
-edit-item = Edit {$type}
-list-item = List {$type}
-add-item = Add {$type}

-user = {$count -> 
    *[one] User
    [other] Users
}

-role = {$count -> 
    *[one] Role
    [other] Roles
}

-permission-statement = {$count -> 
    *[one] Permission Statement
    [other] Permission Statements
}

-created-at = Created At
-permission-statements = Permission Statements
-name = Name

edit-user = {-edit-item(type: {-user})}
list-users = {-list-item(type: {-user(count: 0)})}
save-user = {-save-item(type: {-user})}
saved-user = {-saved-item(type: {-user})}

add-role = {-add-item(type: {-role})}
edit-role = {-edit-item(type: {-role})}
save-role = {-save-item(type: {-role})}
saved-role = {-saved-item(type: {-role})}
list-roles = {-list-item(type: {-role(count: 0)})}

edit-permission-statement = {-edit-item(type: {-permission-statement})}
list-permission-statement = {-list-item(type: {-permission-statement(count: 0)})}
save-permission-statement = {-save-item(type: {-permission-statement})}
saved-permission-statement = {-saved-item(type: {-permission-statement})}

form-field-required = {$field} is required

user-fields-id = {-user(count:1)} Id
user-fields-screenname = Screen Name
user-fields-created-at = {-created-at}
user-fields-assigned-roles = Assigned {-role(count:0)}


role-fields-id = {-role(count:1)} Id
role-fields-name = {-name}
role-fields-created-at = {-created-at}
role-fields-permission-statements = {-permission-statements}

permission-statements-id = {-permission-statement(count:1)} Id
permission-statements-service = Service
permission-statements-resource-type = Resource Type
permission-statements-resource-id = Resource Id
permission-statements-action = Action
permission-statements-allow = Allow / Deny

any-service = Any Service
any-resource-type = Any Resource Type
any-resource-id = All Resources
any-action = Any Action
action-denied = Denied
action-allowed = Allowed