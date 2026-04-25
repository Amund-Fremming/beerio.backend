## TASKS

- WHEN IMPLEMENTING SEPARAION OF CONCERN IS IMPORTATNT
- [ ] dockerfile coompose file with database postgres and pg admin
- [ ] create a foler for all files to be stored.
- [ ] handlers.rs with a router registration on top, have these handlers:
  - [ ] db schema for rom er (room id, user name, enheter) room id må være pk o g unik, username ike unik, men må valdieres på post join så ikke tp har samme navn, indekser på room id, og room id + navn.
  - [ ] health -> returns ok
  - [ ] health/detailed -> returns server: ok, and databse: (do chekc that you can hit the database)
  - [ ] get: room id (string), returns struct with room name, a lsit of players and their sccore.
  - [ ] post: add to room, needs to pass in a name, and valdiate that the name does not exist in that room
  - [ ] post create room: send inn enhet størrelse, 0.33 eller 0.5, og mål fo antall enheter
  - [ ] post: add enhet: sender inn en 0.33 eller 0.55, score lagres som double, så hvis enhet måling er 033 og jeg legger til 0.5 så må score oppdateres til 1.noe så man kan legge inn og få score med komma

  - [ ] cRete tests
  - [ ] run tuntil all tests pass
