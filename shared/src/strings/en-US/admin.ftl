-edit-item = Edit {$type}
-list-item = List {$type}

-user = {$count -> 
    [one] User
    *[other] Users
}

-role = Role

edit-user = {-edit-item(type: {-user(count: 1)})}
list-users = {-list-item(type: {-user(count: 0)})}
edit-role = {-edit-item(type: {-role})}

form-field-required = {$field} is required

user-fields-screenname = Screen Name