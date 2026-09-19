# aws ecs describe-task-definition の taskDefinition から、migrate のイメージだけを $image に替えて
# register-task-definition に渡せる形にする。登録時に受け付けない項目(ARN・リビジョン・状態など)は落とす
.containerDefinitions |= map(if .name == "migrate" then .image = $image else . end)
| {
    family,
    taskRoleArn,
    executionRoleArn,
    networkMode,
    containerDefinitions,
    volumes,
    placementConstraints,
    requiresCompatibilities,
    cpu,
    memory,
    runtimePlatform
  }
| with_entries(select(.value != null))
