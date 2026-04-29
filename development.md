
# Release workflow notes

Tag the repository to build and publish a rotelle-image. Not all tags and
images should be updated to for users.

When evaluated and ready for relase, (this is a separate decision from
publishing), update `k8s/rotelle.yaml`. to reference a specific published
version.

`k8s/rotelle.yaml` is the reference manifest for non-developer users of Rotelle
(which may be users of Infraware terminal evaluating it). Whatever version it
references, will be pulled by users of infraware terminal.

TODO: we will likely also setup a :stable tag which would also have effects
of causing kubernetes clusters already referencing that tag to potentially
pull a new image on the next pod restart.

