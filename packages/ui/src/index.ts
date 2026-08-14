/**
 * @dropvoice/ui — shared UI primitives, composites, and layout for DropVoice.
 *
 * Built on @base-ui/react + class-variance-authority + Tailwind CSS v4 tokens.
 * Apps consume the source directly via the workspace; bring your own Tailwind
 * and import both `@dropvoice/ui/tokens/theme.css` (generated) and
 * `@dropvoice/ui/tokens/effects.css` (hand-written liquid-glass layer).
 */

// Load the Inter webfont (side-effect @font-face imports).
import './fonts';

// Internal utilities.
export { cn } from './lib/cn';

// Primitives.
export { Button, buttonVariants, type ButtonProps, type ButtonVariants } from './primitives/Button';
export { Input, type InputProps } from './primitives/Input';
export { TextArea, type TextAreaProps } from './primitives/TextArea';
export { Select, type SelectProps, type SelectOption } from './primitives/Select';
export { Dialog, type DialogProps } from './primitives/Dialog';
export { Switch, type SwitchProps } from './primitives/Switch';
export { Tooltip, type TooltipProps } from './primitives/Tooltip';
export { Badge, type BadgeProps, type BadgeVariants } from './primitives/Badge';
export { Progress, type ProgressProps } from './primitives/Progress';
export { Alert, type AlertProps, type AlertVariants } from './primitives/Alert';
export {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
  type CardProps,
} from './primitives/Card';

// Composite components.
export { QRCode, type QRCodeProps } from './composite/QRCode';
export { DeviceSelector, type DeviceSelectorProps } from './composite/DeviceSelector';
export { ConnectionStatus, type ConnectionStatusProps } from './composite/ConnectionStatus';
export { Toaster, type ToasterProps, toast } from './composite/Toast';

// Layout components.
export {
  Header,
  type HeaderProps,
  type HeaderLanguageOption,
  type ThemeMode,
} from './layout/Header';
export { PageContainer, type PageContainerProps } from './layout/PageContainer';
export { LiquidBackground, type LiquidBackgroundProps } from './layout/LiquidBackground';

// Hooks.
export { useErrorHandler, type UseErrorHandlerReturn } from './hooks/useErrorHandler';
export { useTheme } from './hooks/useTheme';
