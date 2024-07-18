# daemon

## TODO

- [ ] remove unwraps so this binary is less likely to panic
- [ ] set up systemd failure handling
  - [ ] set up email warning when daemon crashes using Exec, see https://www.redhat.com/sysadmin/systemd-automate-recovery#Take%20action%20on%20failure
  - [ ] run systemctl reset-failed on deploy with github action and just install
- [ ] Set up Leptos app as part of web interface that copies static files to /etc on deploy (or figure out where they should go)
- [ ] implement upload API.
- [ ] Improve forwarding and logging
- [ ] Start on Docker interaction
- [ ] CLI improvements
