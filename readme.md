
read
```graphql
query {
  posts {
    id
    created
  }
}
```
write
```graphql
mutation {
  addPost(postInput: {id: "id1", created: "created1"}) {
    id
    created
  }
}
```