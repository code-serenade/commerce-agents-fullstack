import { useState } from "react"
import {
  AdvancedSelect,
  Badge,
  Calendar,
  Checkbox,
  Combobox,
  Field,
  FieldContent,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSet,
  Input,
  InputGroup,
  InputOTP,
  Label,
  NativeSelect,
  RadioGroup,
  RadioGroupItem,
  SearchInput,
  Select,
  Slider,
  Switch,
  Textarea,
  Time,
  Toggle,
  ToggleGroup,
} from "@workspace/ui-components"
import { DemoBlock, DemoGrid, GuidePage, GuideSection } from "@/components/guide/GuideScaffold"

const TEAM_OPTIONS = [
  { label: "Design Team", value: "design" },
  { label: "Platform Team", value: "platform" },
  { label: "Growth Team", value: "growth" },
]

const CITY_OPTIONS = [
  { label: "Shanghai", value: "shanghai" },
  { label: "Shenzhen", value: "shenzhen" },
  { label: "Tokyo", value: "tokyo" },
]

const PRIORITY_OPTIONS = [
  { label: "P0", value: "p0" },
  { label: "P1", value: "p1" },
  { label: "P2", value: "p2" },
]

export default function FormGuidePage() {
  const [name, setName] = useState("Workspace Portal")
  const [search, setSearch] = useState("billing")
  const [notes, setNotes] = useState(
    "Need approval flow, owner visibility and delivery notes."
  )
  const [checkboxValue, setCheckboxValue] = useState<boolean | "indeterminate">(
    true
  )
  const [switchValue, setSwitchValue] = useState(true)
  const [channel, setChannel] = useState("email")
  const [team, setTeam] = useState("platform")
  const [priority, setPriority] = useState("p1")
  const [city, setCity] = useState("shanghai")
  const [assignee, setAssignee] = useState<string | null>("cathy")
  const [budget, setBudget] = useState("240000")
  const [otp, setOtp] = useState("128640")
  const [score, setScore] = useState([72])
  const [layoutMode, setLayoutMode] = useState("card")
  const [calendarValue, setCalendarValue] = useState<Date | undefined>(
    new Date("2026-04-20T00:00:00")
  )
  const [timeValue, setTimeValue] = useState({
    hour: "09",
    minute: "30",
    second: "00",
  })

  return (
    <GuidePage
      title="Form Components"
      description="这一页集中展示输入、选择、时间与表单结构类组件。示例尽量保持受控写法，方便直接复制到业务表单里。"
      badges={[
        <Badge key="input" variant="outline">
          Input
        </Badge>,
        <Badge key="select" variant="outline">
          Select
        </Badge>,
        <Badge key="picker" variant="outline">
          Picker
        </Badge>,
      ]}
      stats={[
        { label: "Text Input", value: "6" },
        { label: "Selection", value: "8" },
        { label: "Time & Date", value: "2" },
      ]}
    >
      <GuideSection
        id="form-text"
        title="Text Inputs"
        description="基础输入、搜索、输入组、验证码和多行输入放在一个类别里，优先覆盖真实后台页面最常用的写法。"
      >
        <DemoGrid>
          <DemoBlock title="Input / SearchInput" description="基础文本输入和带搜索语义的输入框。">
            <div className="space-y-4">
              <div className="space-y-2">
                <Label>Workspace Name</Label>
                <Input
                  value={name}
                  onValueChange={setName}
                  placeholder="Type workspace name"
                />
              </div>
              <div className="space-y-2">
                <Label>Global Search</Label>
                <SearchInput
                  value={search}
                  onValueChange={setSearch}
                  placeholder="Search orders, users, tickets..."
                  trailingContent={
                    <span className="text-xs text-muted-foreground">
                      Enter to search
                    </span>
                  }
                  updateStrategy="enter"
                />
              </div>
            </div>
          </DemoBlock>

          <DemoBlock title="Textarea / InputGroup" description="适合备注、金额和带单位前后缀的输入场景。">
            <div className="space-y-4">
              <div className="space-y-2">
                <Label>Delivery Notes</Label>
                <Textarea
                  value={notes}
                  onValueChange={setNotes}
                  placeholder="Write notes here"
                  rows={5}
                />
              </div>
              <div className="space-y-2">
                <Label>Budget</Label>
                <InputGroup
                  value={budget}
                  onValueChange={setBudget}
                  prefix="¥"
                  placeholder="0"
                  actionLabel="Sync"
                />
              </div>
            </div>
          </DemoBlock>
        </DemoGrid>

        <DemoGrid>
          <DemoBlock title="InputOTP" description="验证码、支付确认、设备配对都可以直接用这个封装。">
            <div className="space-y-3">
              <InputOTP value={otp} onValueChange={setOtp} length={6} />
              <p className="text-xs text-muted-foreground">Current value: {otp}</p>
            </div>
          </DemoBlock>

          <DemoBlock title="Field / Label" description="Field 适合把 label、描述和错误信息作为一组管理。">
            <FieldSet>
              <FieldLegend>Deployment Policy</FieldLegend>
              <FieldGroup>
                <Field>
                  <Checkbox
                    checked={checkboxValue}
                    onCheckedChange={setCheckboxValue}
                  />
                  <FieldContent>
                    <FieldLabel>Require peer review</FieldLabel>
                    <FieldDescription>
                      Every release must be confirmed by another owner.
                    </FieldDescription>
                    <FieldError>Optional preview state for form feedback.</FieldError>
                  </FieldContent>
                </Field>
              </FieldGroup>
            </FieldSet>
          </DemoBlock>
        </DemoGrid>
      </GuideSection>

      <GuideSection
        id="form-selection"
        title="Selection Controls"
        description="复选、开关、单选、下拉和组合选择器都归到这一组。"
      >
        <DemoGrid>
          <DemoBlock title="Checkbox / Switch / RadioGroup" description="三类最常见的选择交互。">
            <div className="space-y-5">
              <div className="flex items-center gap-3">
                <Checkbox
                  checked={checkboxValue}
                  onCheckedChange={setCheckboxValue}
                />
                <span className="text-sm">Enable invoice review</span>
              </div>
              <div className="flex items-center gap-3">
                <Switch checked={switchValue} onCheckedChange={setSwitchValue} />
                <span className="text-sm">
                  Auto publish: {switchValue ? "On" : "Off"}
                </span>
              </div>
              <RadioGroup value={channel} onValueChange={setChannel}>
                <div className="flex flex-col gap-3">
                  <RadioGroupItem value="email">Email</RadioGroupItem>
                  <RadioGroupItem value="sms">SMS</RadioGroupItem>
                  <RadioGroupItem value="webhook">Webhook</RadioGroupItem>
                </div>
              </RadioGroup>
            </div>
          </DemoBlock>

          <DemoBlock title="Select Family" description="统一展示 Select、AdvancedSelect、NativeSelect 和 Combobox。">
            <div className="space-y-4">
              <div className="space-y-2">
                <Label>Team</Label>
                <Select
                  value={team}
                  onValueChange={setTeam}
                  list={TEAM_OPTIONS}
                  placeholder="Select team"
                />
              </div>
              <div className="space-y-2">
                <Label>Priority</Label>
                <AdvancedSelect
                  value={priority}
                  onValueChange={setPriority}
                  list={PRIORITY_OPTIONS}
                  placeholder="Select priority"
                  allowClear
                />
              </div>
              <div className="space-y-2">
                <Label>City</Label>
                <NativeSelect
                  value={city}
                  onValueChange={setCity}
                  options={CITY_OPTIONS}
                  placeholder="Choose a city"
                />
              </div>
              <div className="space-y-2">
                <Label>Assignee</Label>
                <Combobox
                  value={assignee}
                  onValueChange={setAssignee}
                  options={[
                    { label: "Alice", value: "alice" },
                    { label: "Bob", value: "bob" },
                    { label: "Cathy", value: "cathy" },
                    { label: "David", value: "david" },
                  ]}
                  placeholder="Search member..."
                />
              </div>
            </div>
          </DemoBlock>
        </DemoGrid>

        <DemoGrid>
          <DemoBlock title="Slider" description="数值调节类组件，适合评分、阈值、预算比例。">
            <div className="space-y-4">
              <Slider
                value={score}
                onValueChange={setScore}
                min={0}
                max={100}
                step={1}
              />
              <p className="text-sm text-muted-foreground">
                Current quality score: {score[0]}
              </p>
            </div>
          </DemoBlock>

          <DemoBlock title="Toggle / ToggleGroup" description="适合视图切换、过滤标签和二元开关。">
            <div className="space-y-4">
              <Toggle
                pressed={switchValue}
                onPressedChange={setSwitchValue}
                label={switchValue ? "Pinned" : "Pin panel"}
              />
              <ToggleGroup
                value={layoutMode}
                onValueChange={(value) => {
                  if (value) {
                    setLayoutMode(value)
                  }
                }}
                options={[
                  { label: "Card", value: "card" },
                  { label: "Table", value: "table" },
                  { label: "Board", value: "board" },
                ]}
              />
            </div>
          </DemoBlock>
        </DemoGrid>
      </GuideSection>

      <GuideSection
        id="form-picker"
        title="Date & Time"
        description="日期与时间封装用法保持独立，便于业务侧单独拎出来复用。"
      >
        <DemoGrid>
          <DemoBlock title="Calendar" description="支持 single / multiple / range，这里用 single 模式做基础示例。">
            <div className="space-y-3">
              <Calendar
                mode="single"
                value={calendarValue}
                onValueChange={(value) => {
                  if (value instanceof Date || value === undefined) {
                    setCalendarValue(value)
                  }
                }}
                locale="zhCN"
              />
              <p className="text-xs text-muted-foreground">
                Selected date:{" "}
                {calendarValue
                  ? calendarValue.toLocaleDateString("zh-CN")
                  : "None"}
              </p>
            </div>
          </DemoBlock>

          <DemoBlock title="Time" description="支持秒级显示和尺寸切换，适合时间点选择。">
            <div className="space-y-3">
              <Time
                value={timeValue}
                onValueChange={(value) =>
                  setTimeValue({
                    hour: value.hour,
                    minute: value.minute,
                    second: value.second ?? "00",
                  })
                }
                showSeconds
                size="md"
              />
              <p className="text-xs text-muted-foreground">
                Current time: {timeValue.hour}:{timeValue.minute}:
                {timeValue.second}
              </p>
            </div>
          </DemoBlock>
        </DemoGrid>
      </GuideSection>
    </GuidePage>
  )
}
