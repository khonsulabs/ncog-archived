-edit-item = Edit {$type}
-list-item = List {$type}

-user = {$count -> 
    [one] User
    *[other] Users
}

-role = {$count -> 
    [one] Role
    *[other] Roles
}

-created-at = Created At
-name = Name

edit-user = {-edit-item(type: {-user(count: 1)})}
list-users = {-list-item(type: {-user(count: 0)})}
edit-role = {-edit-item(type: {-role})}
list-roles = {-list-item(type: {-role(count: 0)})}

form-field-required = {$field} is required

user-fields-id = {-user(count:1)} Id
user-fields-screenname = Screen Name
user-fields-created-at = {-created-at}
user-fields-assigned-roles = Assigned {-role(count:0)}


role-fields-id = {-role(count:1)} Id
role-fields-name = {-name}
role-fields-created-at = {-created-at}