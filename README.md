# sailor

Application deployment that is easy, reliable and self-owned.

## TODO
- [ ] set up systemd failure handling
  - [ ] set up email warning when daemon crashes using Exec, see https://www.redhat.com/sysadmin/systemd-automate-recovery#Take%20action%20on%20failure
  - [ ] run systemctl reset-failed on deploy with github action and just install
- [ ] Implement graceful hyper shutdown for sailor daemon server
- [ ] Set up Leptos for web crate
    - Don't forget to update Just and CI build scripts to run `cargo leptos build`
    - Don't forget to update CI upgrade script to copy over new `public` assets folder.
- [ ] Implement upload endpoint for Docker images
    - Allow secret key creation during app creation that can be rotated with the CLI. This secret must be provided when uploading an app as a security/protection feature.
- [ ] Set up Docker image for `reading.jensmeindertsma.com`
- [ ] Set up CI for `reading.jensmeindertsma.com` that uploads to the endpoint
- [ ] Implement Docker container restarting when new images are uploaded
- [ ] Implement more CLI commands that give an overview of current status