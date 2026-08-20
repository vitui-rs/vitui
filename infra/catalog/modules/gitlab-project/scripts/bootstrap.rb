# Creates, idempotently, the two things a host repository needs in order to push here and watch a
# pipeline run: a project under `root`, and a personal access token to push with.
#
# Run through `gitlab-rails runner`, which is the only interface this instance has before a token
# exists — the REST API is exactly what we are bootstrapping our way into.

user = User.find_by_username('root')
raise 'no root user; GitLab has not finished its first boot' if user.nil?

path = ENV.fetch('PROJECT_PATH')
full_path = "#{user.username}/#{path}"

project = Project.find_by_full_path(full_path)
if project.nil?
  project = ::Projects::CreateService.new(
    user,
    name: path,
    path: path,
    namespace_id: user.namespace_id,
    visibility_level: Gitlab::VisibilityLevel::PRIVATE,
    initialize_with_readme: false
  ).execute
  raise "project creation failed: #{project.errors.full_messages.join(', ')}" if project.errors.any?
end

# A personal access token's plaintext exists only in the process that created it — the record keeps
# a digest. So an existing token cannot be re-read, only replaced. Revoking and reissuing is the
# idempotent shape available, and on a throwaway local instance it costs nothing.
name = ENV.fetch('TOKEN_NAME')
user.personal_access_tokens.active.where(name: name).find_each(&:revoke!)

token = user.personal_access_tokens.create!(
  name: name,
  scopes: %w[api write_repository],
  expires_at: 300.days.from_now
)

File.write(ENV.fetch('OUT_PROJECT_PATH'), project.full_path)
File.write(ENV.fetch('OUT_TOKEN'), token.token)
